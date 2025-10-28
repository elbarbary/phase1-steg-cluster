// Configuration
let clusterNodes = [];
let statusPollInterval = 1000;
let currentStressTest = null;
let stegoImageData = null;
let recoveredImageData = null;

// Charts
let throughputChart = null;
let latencyChart = null;

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
    // Try default local nodes
    const defaultNodes = [
        'http://127.0.0.1:8081',
        'http://127.0.0.1:8082',
        'http://127.0.0.1:8083',
        'http://10.0.0.11:8081',
        'http://10.0.0.12:8082',
        'http://10.0.0.13:8083',
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
// Load Balancing
// ============================================================================

async function selectBestNode() {
    if (clusterNodes.length === 0) {
        // Fallback to localhost
        return 'http://127.0.0.1:8081';
    }
    
    // Filter healthy nodes
    const healthy = clusterNodes.filter(n => n.healthy);
    if (healthy.length === 0) {
        throw new Error('No healthy nodes available');
    }
    
    // Compute scores: lower is better
    const scored = healthy.map(n => {
        const normalizedP95 = n.p95_ms / 100.0; // Normalize to ~0-1 range
        const normalizedQps = n.qps_1m / 10.0;  // Normalize
        const score = 0.6 * n.cpu_pct + 0.3 * normalizedP95 + 0.1 * normalizedQps;
        return { node: n, score };
    });
    
    // Select node with minimum score
    scored.sort((a, b) => a.score - b.score);
    const best = scored[0].node;
    
    return `http://${best.ip}:${best.http_port}`;
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
    
    document.getElementById('start-stress').classList.add('hidden');
    document.getElementById('stop-stress').classList.remove('hidden');
    document.getElementById('stress-stats').classList.remove('hidden');
    
    currentStressTest = {
        running: true,
        totalRequests: numClients * reqsPerClient,
        completed: 0,
        success: 0,
        failure: 0,
        startTime: Date.now(),
        latencies: []
    };
    
    // Reset charts
    resetCharts();
    
    // Start workers
    const promises = [];
    for (let i = 0; i < numClients; i++) {
        promises.push(stressWorker(reqsPerClient, operation));
    }
    
    // Update UI periodically
    const uiInterval = setInterval(() => {
        if (!currentStressTest.running) {
            clearInterval(uiInterval);
            return;
        }
        updateStressUI();
    }, 1000);
    
    // Wait for completion
    await Promise.all(promises);
    
    currentStressTest.running = false;
    clearInterval(uiInterval);
    updateStressUI();
    
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
    for (let i = 0; i < numRequests; i++) {
        if (!currentStressTest.running) break;
        
        const start = performance.now();
        let success = false;
        
        try {
            const node = await selectBestNode();
            const datasetIdx = Math.floor(Math.random() * 50);
            
            // Fetch dataset image
            const imgResponse = await fetch(`${node}/api/dataset/${datasetIdx}`);
            const imgBlob = await imgResponse.blob();
            const imgFile = new File([imgBlob], 'test.png', { type: 'image/png' });
            
            // Perform operation
            if (operation === 'embed') {
                await embedSecret(node, imgFile);
            } else {
                await extractSecret(node, imgFile);
            }
            
            success = true;
        } catch (error) {
            // Failed request
        }
        
        const latency = performance.now() - start;
        
        currentStressTest.completed++;
        if (success) {
            currentStressTest.success++;
            currentStressTest.latencies.push(latency);
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
    updateCharts(throughput, test.latencies);
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
}

function resetCharts() {
    throughputChart.data.labels = [];
    throughputChart.data.datasets[0].data = [];
    throughputChart.update();
    
    latencyChart.data.labels = [];
    latencyChart.data.datasets[0].data = [];
    latencyChart.data.datasets[1].data = [];
    latencyChart.update();
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
