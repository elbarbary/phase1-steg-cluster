// Configuration
let clusterNodes = [];
let statusPollInterval = 1000;
let currentStressTest = null;
let stegoImageData = null;
let recoveredImageData = null;

// Charts
let throughputChart = null;
let latencyChart = null;
let distributionChart = null;

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    initTabs();
    initEmbedHandler();
    initExtractHandler();
    initClusterStatus();
    initStressTest();
    initCharts();
});

// ============================================================================
// Tab Management
// ============================================================================

function initTabs() {
    const tabButtons = document.querySelectorAll('.tab-button');
    tabButtons.forEach(button => {
        button.addEventListener('click', () => {
            const targetTab = button.dataset.tab;
            
            // Update active states
            tabButtons.forEach(b => b.classList.remove('active'));
            button.classList.add('active');
            
            document.querySelectorAll('.tab-content').forEach(content => {
                content.classList.remove('active');
            });
            document.getElementById(`${targetTab}-tab`).classList.add('active');
        });
    });
}

// ============================================================================
// Steganography - Embed
// ============================================================================

function initEmbedHandler() {
    const fileInput = document.getElementById('embed-file');
    const embedBtn = document.getElementById('embed-btn');
    
    fileInput.addEventListener('change', (e) => {
        if (e.target.files.length > 0) {
            const file = e.target.files[0];
            const reader = new FileReader();
            reader.onload = (event) => {
                document.getElementById('original-preview').src = event.target.result;
                document.getElementById('original-size').textContent = formatBytes(file.size);
            };
            reader.readAsDataURL(file);
        }
    });
    
    embedBtn.addEventListener('click', async () => {
        const file = fileInput.files[0];
        if (!file) {
            alert('Please select a file');
            return;
        }
        
        embedBtn.disabled = true;
        embedBtn.textContent = 'Processing...';
        
        try {
            const node = await selectBestNode();
            const result = await embedSecret(node, file);
            displayEmbedResult(result);
        } catch (error) {
            alert(`Embed failed: ${error.message}`);
        } finally {
            embedBtn.disabled = false;
            embedBtn.textContent = 'Embed';
        }
    });
    
    document.getElementById('download-stego').addEventListener('click', () => {
        if (stegoImageData) {
            downloadFile(stegoImageData, 'stego.png', 'image/png');
        }
    });
}

async function embedSecret(nodeUrl, file) {
    const formData = new FormData();
    formData.append('file', file);
    
    const response = await fetch(`${nodeUrl}/api/embed`, {
        method: 'POST',
        body: formData
    });
    
    if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Embed failed');
    }
    
    return await response.json();
}

function displayEmbedResult(result) {
    document.getElementById('embed-result').classList.remove('hidden');
    
    stegoImageData = `data:image/png;base64,${result.stego_image_b64}`;
    document.getElementById('stego-preview').src = stegoImageData;
    document.getElementById('stego-info').textContent = 
        `${result.cover_info.width}×${result.cover_info.height}, ${formatBytes(result.payload_size_bytes)}`;
    
    document.getElementById('embed-request-id').textContent = result.request_id;
    document.getElementById('embed-secret-size').textContent = formatBytes(result.secret_size_bytes);
    document.getElementById('embed-capacity').textContent = 
        `${formatBytes(result.cover_info.capacity_bytes)} (${result.cover_info.lsb_per_channel} LSB)`;
}

// ============================================================================
// Steganography - Extract
// ============================================================================

function initExtractHandler() {
    const fileInput = document.getElementById('extract-file');
    const extractBtn = document.getElementById('extract-btn');
    
    extractBtn.addEventListener('click', async () => {
        const file = fileInput.files[0];
        if (!file) {
            alert('Please select a stego image');
            return;
        }
        
        extractBtn.disabled = true;
        extractBtn.textContent = 'Processing...';
        
        try {
            const node = await selectBestNode();
            const result = await extractSecret(node, file);
            displayExtractResult(result);
        } catch (error) {
            alert(`Extract failed: ${error.message}`);
        } finally {
            extractBtn.disabled = false;
            extractBtn.textContent = 'Extract';
        }
    });
    
    document.getElementById('download-recovered').addEventListener('click', () => {
        if (recoveredImageData) {
            downloadFile(recoveredImageData, 'recovered.png', 'image/png');
        }
    });
}

async function extractSecret(nodeUrl, file) {
    const formData = new FormData();
    formData.append('file', file);
    
    const response = await fetch(`${nodeUrl}/api/extract`, {
        method: 'POST',
        body: formData
    });
    
    if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Extract failed');
    }
    
    return await response.json();
}

function displayExtractResult(result) {
    document.getElementById('extract-result').classList.remove('hidden');
    
    recoveredImageData = `data:${result.recovered_mime};base64,${result.recovered_b64}`;
    
    if (result.recovered_mime.startsWith('image/')) {
        document.getElementById('recovered-preview').src = recoveredImageData;
    } else {
        document.getElementById('recovered-preview').alt = 'Not an image';
    }
    
    document.getElementById('recovered-info').textContent = formatBytes(result.recovered_size_bytes);
    document.getElementById('extract-request-id').textContent = result.request_id;
    document.getElementById('extract-size').textContent = formatBytes(result.recovered_size_bytes);
    document.getElementById('extract-mime').textContent = result.recovered_mime;
}

// ============================================================================
// Cluster Status
// ============================================================================

function initClusterStatus() {
    updateClusterStatus();
    setInterval(updateClusterStatus, statusPollInterval);
}

async function updateClusterStatus() {
    try {
        // Try to query any available node
        const nodes = await discoverNodes();
        if (nodes.length === 0) {
            console.warn('No nodes available');
            return;
        }
        
        const status = await fetchClusterStatus(nodes[0]);
        displayClusterStatus(status);
        clusterNodes = status.nodes;
    } catch (error) {
        console.error('Failed to update cluster status:', error);
    }
}

async function discoverNodes() {
    // Try cluster nodes - use actual IPs from config
    // This gets called from browser, so use real cluster IPs
    const defaultNodes = [
        'http://172.20.10.2:8081',  // n1
        'http://172.20.10.3:8082',  // n2
        'http://172.20.10.6:8083',  // n3
        'http://127.0.0.1:8081',    // localhost (if running locally)
    ];
    
    const available = [];
    for (const node of defaultNodes) {
        try {
            const response = await fetch(`${node}/healthz`, { 
                method: 'GET',
                signal: AbortSignal.timeout(1000)
            });
            if (response.ok) {
                available.push(node);
            }
        } catch (e) {
            // Node not available
        }
    }
    
    return available;
}

async function fetchClusterStatus(nodeUrl) {
    const response = await fetch(`${nodeUrl}/cluster/status`);
    if (!response.ok) {
        throw new Error('Failed to fetch cluster status');
    }
    return await response.json();
}

function displayClusterStatus(status) {
    document.getElementById('cluster-term').textContent = status.term;
    
    const leaderBadge = document.getElementById('cluster-leader');
    leaderBadge.textContent = status.leader_id || 'None';
    leaderBadge.className = status.leader_id ? 'badge badge-success' : 'badge badge-warning';
    
    const tbody = document.getElementById('node-table-body');
    tbody.innerHTML = '';
    
    status.nodes.forEach(node => {
        const row = document.createElement('tr');
        row.innerHTML = `
            <td>${node.id}</td>
            <td><span class="badge ${node.role === 'Leader' ? 'badge-primary' : 'badge-secondary'}">${node.role}</span></td>
            <td>${node.ip}:${node.http_port}</td>
            <td>${node.cpu_pct.toFixed(1)}%</td>
            <td>${node.mem_pct.toFixed(1)}%</td>
            <td>${node.qps_1m.toFixed(2)}</td>
            <td>${node.p95_ms.toFixed(2)}</td>
            <td><span class="badge ${node.healthy ? 'badge-success' : 'badge-danger'}">${node.healthy ? 'Healthy' : 'Down'}</span></td>
            <td>
                <button class="btn btn-sm btn-danger" onclick="failNode('${node.id}', '${node.ip}', ${node.http_port})">Fail</button>
                <button class="btn btn-sm btn-success" onclick="restoreNode('${node.id}', '${node.ip}', ${node.http_port})">Restore</button>
            </td>
        `;
        tbody.appendChild(row);
    });
}

async function failNode(nodeId, ip, port) {
    const action = confirm(`Fail node ${nodeId}? Choose:\nOK = Crash (exit process)\nCancel = Pause (reject requests)`);
    const actionType = action ? 'crash' : 'pause';
    
    try {
        const response = await fetch(`http://${ip}:${port}/admin/fail`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ action: actionType })
        });
        
        if (actionType === 'pause' && response.ok) {
            alert(`Node ${nodeId} paused`);
        }
        // If crash, the node won't respond
    } catch (error) {
        if (actionType === 'crash') {
            alert(`Node ${nodeId} crashed (expected)`);
        } else {
            alert(`Failed to pause node: ${error.message}`);
        }
    }
}

async function restoreNode(nodeId, ip, port) {
    try {
        const response = await fetch(`http://${ip}:${port}/admin/restore`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' }
        });
        
        if (response.ok) {
            alert(`Node ${nodeId} restored`);
        }
    } catch (error) {
        alert(`Failed to restore node: ${error.message}`);
    }
}

// ============================================================================
// Load Balancing (Raft-based with Leader Routing)
// ============================================================================

// Cache for cluster state
let nodeBalancingCache = {
    lastFetch: 0,
    leader: null,
    nodes: [],
    cacheDuration: 5000  // 5 second cache
};

/**
 * Get cluster status and discover the leader
 * This fetches from any available node to get full cluster view
 */
async function getClusterLeaderAndNodes() {
    const now = Date.now();
    
    // Return cached result if still fresh
    if (nodeBalancingCache.lastFetch > 0 && now - nodeBalancingCache.lastFetch < nodeBalancingCache.cacheDuration) {
        return {
            leader: nodeBalancingCache.leader,
            nodes: nodeBalancingCache.nodes
        };
    }
    
    // Discover available nodes
    const availableNodes = await discoverNodes();
    if (availableNodes.length === 0) {
        throw new Error('No nodes available in cluster');
    }
    
    // Query first available node for cluster status
    try {
        const status = await fetchClusterStatus(availableNodes[0]);
        
        // Extract leader and running nodes
        const leader = status.leader_id;
        const nodes = status.nodes
            .filter(n => n.healthy)  // Only use healthy nodes
            .map(n => ({
                id: n.id,
                url: `http://${n.ip}:${n.http_port}`,
                role: n.role,
                qps: n.qps_1m
            }));
        
        // Cache the result
        nodeBalancingCache = {
            lastFetch: now,
            leader,
            nodes,
            cacheDuration: 5000
        };
        
        return { leader, nodes };
    } catch (error) {
        console.error('Failed to fetch cluster status for load balancing:', error);
        throw error;
    }
}

/**
 * Select best node for request using Raft-based load balancing
 * Strategy:
 * 1. Primary: Route through LEADER for consistency and safety
 * 2. Fallback: If leader unavailable, use least-loaded healthy node
 */
async function selectBestNode() {
    try {
        const { leader, nodes } = await getClusterLeaderAndNodes();
        
        if (!leader || leader === '') {
            // No leader elected, use least-loaded node
            if (nodes.length === 0) {
                throw new Error('No healthy nodes available');
            }
            // Sort by QPS and pick least loaded
            nodes.sort((a, b) => a.qps - b.qps);
            return nodes[0].url;
        }
        
        // Find leader URL
        const leaderNode = nodes.find(n => n.id === leader);
        if (leaderNode) {
            return leaderNode.url;
        }
        
        // Leader not in nodes list (shouldn't happen), use any node
        if (nodes.length > 0) {
            nodes.sort((a, b) => a.qps - b.qps);
            return nodes[0].url;
        }
        
        throw new Error('No valid nodes found for load balancing');
    } catch (error) {
        // Fallback to current origin if load balancing fails
        console.warn('Load balancing failed, using current origin:', error.message);
        return window.location.origin;
    }
}

// Attempt a request with automatic retry on failure to handle failover
async function requestWithFailover(nodeUrl, requestFn, maxRetries = 2) {
    let lastError = null;
    const availableNodes = [];
    
    try {
        const { nodes } = await getClusterLeaderAndNodes();
        for (const node of nodes) {
            availableNodes.push(node.url);
        }
    } catch (e) {
        // Ignore error getting nodes list
    }
    
    // Remove duplicate nodeUrl
    const uniqueNodes = [nodeUrl, ...availableNodes.filter(n => n !== nodeUrl)];
    
    for (let attempt = 0; attempt < uniqueNodes.length && attempt <= maxRetries; attempt++) {
        const tryNode = uniqueNodes[attempt];
        try {
            return await requestFn(tryNode);
        } catch (error) {
            lastError = error;
            // If leader seems down, refresh cluster state and try next node
            if (attempt < uniqueNodes.length - 1) {
                await new Promise(r => setTimeout(r, 50)); // Brief delay before retry
            }
        }
    }
    
    throw lastError || new Error('All failover attempts exhausted');
}

// ============================================================================
// Stress Testing
// ============================================================================

function initStressTest() {
    document.getElementById('start-stress').addEventListener('click', startStressTest);
    document.getElementById('stop-stress').addEventListener('click', stopStressTest);
}

async function startStressTest() {
    const numClients = parseInt(document.getElementById('num-clients').value);
    const reqsPerClient = parseInt(document.getElementById('reqs-per-client').value);
    const operation = document.getElementById('operation').value;
    
    const totalRequests = numClients * reqsPerClient;
    
    // Validation: prevent accidental overload
    if (totalRequests > 500000) {
        alert(`⚠️ Too many requests! Max 500,000 total (you set ${totalRequests}).\n\nTry: ${Math.floor(500000 / numClients)} reqs/client or ${Math.floor(500000 / reqsPerClient)} clients`);
        return;
    }
    
    if (totalRequests > 10000) {
        const confirm_large = confirm(`⚠️ Large test: ${totalRequests} total requests. This will take a while. Continue?`);
        if (!confirm_large) return;
    }
    
    document.getElementById('start-stress').classList.add('hidden');
    document.getElementById('stop-stress').classList.remove('hidden');
    document.getElementById('stress-stats').classList.remove('hidden');
    
    // NEW: Get cluster info before starting
    let clusterInfo = '';
    try {
        const { leader, nodes } = await getClusterLeaderAndNodes();
        clusterInfo = `Leader: ${leader || 'NONE'} | Active nodes: ${nodes.length}`;
        console.log(`🎯 Starting stress test routed to ${clusterInfo}`);
    } catch (error) {
        console.warn('Could not get cluster info:', error);
        clusterInfo = 'Cluster info unavailable';
    }
    
    currentStressTest = {
        running: true,
        totalRequests: numClients * reqsPerClient,
        completed: 0,
        success: 0,
        failure: 0,
        startTime: Date.now(),
        // Use a reservoir sample for approximate percentiles (memory-efficient)
        latencyReservoir: [],
        reservoirSize: 5000, // Keep 5000 samples for accurate percentiles even at 500k requests
        latencyCount: 0,
        // NEW: Track which nodes received requests (for load balancing verification)
        nodeDistribution: {}
    };
    
    // Reset charts
    resetCharts();
    
    // UPDATED: numClients = number of concurrent workers (direct mapping)
    // Each worker will make reqsPerClient requests sequentially
    // The leader handles load balancing across cluster nodes
    console.log(`🚀 Starting ${numClients} concurrent workers (each making ${reqsPerClient} requests)`);
    console.log(`📊 ${clusterInfo}`);
    console.log(`📈 Total requests: ${totalRequests}`);
    
    // Create all clients as concurrent workers
    const workerPromises = [];
    for (let clientId = 0; clientId < numClients; clientId++) {
        workerPromises.push(
            stressWorker(reqsPerClient, operation)
        );
    }
    
    // Update UI periodically while workers run
    const uiInterval = setInterval(() => {
        if (!currentStressTest.running) {
            clearInterval(uiInterval);
            return;
        }
        updateStressUI();
    }, 1000);
    
    // Wait for all workers to complete
    await Promise.all(workerPromises);
    
    currentStressTest.running = false;
    clearInterval(uiInterval);
    updateStressUI();
    
    // NEW: Show node distribution stats
    if (Object.keys(currentStressTest.nodeDistribution).length > 0) {
        console.log('📈 Load Distribution Across Nodes:');
        for (const [node, count] of Object.entries(currentStressTest.nodeDistribution)) {
            const pct = ((count / currentStressTest.completed) * 100).toFixed(1);
            console.log(`   ${node}: ${count} requests (${pct}%)`);
        }
    }
    
    document.getElementById('start-stress').classList.remove('hidden');
    document.getElementById('stop-stress').classList.add('hidden');
    
    alert('Stress test completed!');
}

function stopStressTest() {
    if (currentStressTest) {
        currentStressTest.running = false;
    }
}

async function stressWorker(numRequests, operation) {
    // Each worker makes sequential requests (no internal queueing)
    // The Raft leader handles load balancing across cluster nodes
    
    for (let requestNum = 0; requestNum < numRequests; requestNum++) {
        if (!currentStressTest.running) break;
        
        const start = performance.now();
        let success = false;
        let selectedNode = null;
        
        try {
            // Get the node being used (routed through Raft leader)
            selectedNode = await selectBestNode();
            const datasetIdx = Math.floor(Math.random() * 50);
            
            // Fetch dataset image with failover
            const imgBlob = await requestWithFailover(selectedNode, async (node) => {
                const imgResponse = await fetch(`${node}/api/dataset/${datasetIdx}`, {
                    timeout: 5000
                });
                if (!imgResponse.ok) {
                    throw new Error(`Dataset fetch failed: ${imgResponse.status}`);
                }
                return await imgResponse.blob();
            });
            
            const imgFile = new File([imgBlob], 'test.png', { type: 'image/png' });
            
            // Perform operation with failover
            await requestWithFailover(selectedNode, async (node) => {
                if (operation === 'embed') {
                    await embedSecret(node, imgFile);
                } else {
                    await extractSecret(node, imgFile);
                }
            });
            
            success = true;
        } catch (error) {
            // Failed request - log for debugging
            console.error(`Request ${requestNum} failed:`, error.message || error);
        }
        
        const latency = performance.now() - start;
        
        currentStressTest.completed++;
        
        // Track which node received this request
        if (selectedNode) {
            if (!currentStressTest.nodeDistribution[selectedNode]) {
                currentStressTest.nodeDistribution[selectedNode] = 0;
            }
            currentStressTest.nodeDistribution[selectedNode]++;
        }
        
        if (success) {
            currentStressTest.success++;
            // Use reservoir sampling to keep memory bounded
            const reservoir = currentStressTest.latencyReservoir;
            if (reservoir.length < currentStressTest.reservoirSize) {
                reservoir.push(latency);
            } else {
                // Replace random element with probability reservoirSize / (count + 1)
                const idx = Math.floor(Math.random() * ++currentStressTest.latencyCount);
                if (idx < currentStressTest.reservoirSize) {
                    reservoir[idx] = latency;
                }
            }
        } else {
            currentStressTest.failure++;
        }
    }
}

function updateStressUI() {
    const test = currentStressTest;
    if (!test) return;
    
    document.getElementById('total-requests').textContent = test.completed;
    document.getElementById('success-count').textContent = test.success;
    document.getElementById('failure-count').textContent = test.failure;
    
    const elapsed = (Date.now() - test.startTime) / 1000;
    const throughput = test.completed / elapsed;
    document.getElementById('throughput').textContent = throughput.toFixed(2);
    
    // Update charts
    updateCharts(throughput, test.latencyReservoir);
    
    // Update distribution chart
    updateDistributionChart(test.nodeDistribution, elapsed);
}

// ============================================================================
// Charts
// ============================================================================

function initCharts() {
    const throughputCtx = document.getElementById('throughput-chart').getContext('2d');
    throughputChart = new Chart(throughputCtx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [{
                label: 'Throughput (req/s)',
                data: [],
                borderColor: 'rgb(75, 192, 192)',
                tension: 0.1
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                y: { beginAtZero: true }
            }
        }
    });
    
    const latencyCtx = document.getElementById('latency-chart').getContext('2d');
    latencyChart = new Chart(latencyCtx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [
                {
                    label: 'P50 Latency (ms)',
                    data: [],
                    borderColor: 'rgb(54, 162, 235)',
                    tension: 0.1
                },
                {
                    label: 'P95 Latency (ms)',
                    data: [],
                    borderColor: 'rgb(255, 99, 132)',
                    tension: 0.1
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                y: { beginAtZero: true }
            }
        }
    });
    
    const distributionCtx = document.getElementById('distribution-chart').getContext('2d');
    distributionChart = new Chart(distributionCtx, {
        type: 'bar',
        data: {
            labels: [],
            datasets: [{
                label: 'Requests/s per Server',
                data: [],
                backgroundColor: [
                    'rgba(255, 99, 132, 0.7)',
                    'rgba(54, 162, 235, 0.7)',
                    'rgba(75, 192, 192, 0.7)'
                ],
                borderColor: [
                    'rgb(255, 99, 132)',
                    'rgb(54, 162, 235)',
                    'rgb(75, 192, 192)'
                ],
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                y: { 
                    beginAtZero: true,
                    title: {
                        display: true,
                        text: 'Requests/s'
                    }
                }
            },
            plugins: {
                title: {
                    display: true,
                    text: 'Request Distribution Across Servers'
                }
            }
        }
    });
}

function resetCharts() {
    throughputChart.data.labels = [];
    throughputChart.data.datasets[0].data = [];
    throughputChart.update();
    
    latencyChart.data.labels = [];
    latencyChart.data.datasets[0].data = [];
    latencyChart.data.datasets[1].data = [];
    latencyChart.update();
    
    distributionChart.data.labels = [];
    distributionChart.data.datasets[0].data = [];
    distributionChart.update();
}

function updateCharts(throughput, latencies) {
    const timestamp = new Date().toLocaleTimeString();
    
    // Throughput chart
    throughputChart.data.labels.push(timestamp);
    throughputChart.data.datasets[0].data.push(throughput);
    
    if (throughputChart.data.labels.length > 60) {
        throughputChart.data.labels.shift();
        throughputChart.data.datasets[0].data.shift();
    }
    
    throughputChart.update('none');
    
    // Latency chart
    if (latencies.length > 0) {
        const sorted = [...latencies].sort((a, b) => a - b);
        const p50 = sorted[Math.floor(sorted.length * 0.5)];
        const p95 = sorted[Math.floor(sorted.length * 0.95)];
        
        latencyChart.data.labels.push(timestamp);
        latencyChart.data.datasets[0].data.push(p50);
        latencyChart.data.datasets[1].data.push(p95);
        
        if (latencyChart.data.labels.length > 60) {
            latencyChart.data.labels.shift();
            latencyChart.data.datasets[0].data.shift();
            latencyChart.data.datasets[1].data.shift();
        }
        
        latencyChart.update('none');
    }
}

function updateDistributionChart(nodeDistribution, elapsedSeconds) {
    if (!nodeDistribution || Object.keys(nodeDistribution).length === 0) {
        return;
    }
    
    // Calculate req/s for each node
    const labels = [];
    const data = [];
    
    for (const [nodeUrl, count] of Object.entries(nodeDistribution)) {
        // Extract node identifier (e.g., "n1", "n2", "n3" or port number)
        let nodeLabel = nodeUrl;
        const match = nodeUrl.match(/172\.20\.10\.(\d+):(\d+)/);
        if (match) {
            const port = match[2];
            const portMap = { '8081': 'n1', '8082': 'n2', '8083': 'n3' };
            nodeLabel = portMap[port] || `Port ${port}`;
        }
        
        labels.push(nodeLabel);
        data.push((count / elapsedSeconds).toFixed(2));
    }
    
    distributionChart.data.labels = labels;
    distributionChart.data.datasets[0].data = data;
    distributionChart.update('none');
}

// ============================================================================
// Utilities
// ============================================================================

function formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

function downloadFile(dataUrl, filename, mimeType) {
    const link = document.createElement('a');
    link.href = dataUrl;
    link.download = filename;
    link.click();
}
