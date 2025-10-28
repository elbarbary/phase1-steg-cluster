# 📤 HOW TO PUSH TO GITHUB - STEP BY STEP

## 🎯 Quick Overview

1. Create GitHub repository
2. Push code from Device 1
3. Other devices clone the repo
4. All devices configured and running

---

## ✅ STEP 1: Create GitHub Account & Repository

### If you don't have a GitHub account:

1. Go to https://github.com/signup
2. Create an account
3. Verify email

### Create Repository

1. Go to https://github.com/new
2. Fill in:
   - **Repository name:** `phase1-steg-cluster`
   - **Description:** `Distributed Steganography with OpenRaft Consensus`
   - **Visibility:** `Public` (so you can share the link)
   - **Initialize repository:** ❌ DO NOT check "Add README" (we already have one)

3. Click **"Create repository"**

You'll see a page with your repository URL like:
```
https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
```

---

## 📤 STEP 2: Push from Device 1 (This Computer)

### Option A: Using HTTPS (Easier for First-Time)

```bash
cd /home/youssef-mansour@auc.egy/dist/phase1-steg-cluster

# Add remote (replace YOUR_USERNAME with your GitHub username)
git remote add origin https://github.com/YOUR_USERNAME/phase1-steg-cluster.git

# Push to GitHub
git push -u origin master
```

When prompted for password:
- **Username:** Your GitHub username
- **Password:** Your GitHub personal access token (NOT your GitHub password!)

### Create Personal Access Token

1. Go to https://github.com/settings/tokens/new
2. Fill in:
   - **Token name:** `phase1-push`
   - **Expiration:** 90 days
   - **Scopes:** Check `repo` (full control of private repositories)
3. Click "Generate token"
4. **Copy the token** (you won't see it again!)
5. Use this token as your password in the git push command

### Push Command

```bash
git push -u origin master
```

**Expected output:**
```
Enumerating objects: 36, done.
Counting objects: 100% (36/36), done.
Delta compression using up to 8 threads
Compressing objects: 100% (30/30), done.
Writing objects: 100% (36/36), 87.23 KiB | 1.23 MiB/s, done.
Total 36 (delta 0), reused 0 (delta 0), pack-reused 0
To https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
 * [new branch]      master -> master
Branch 'master' is set to track remote branch 'master' from 'origin'.
```

**✅ Success!** Your code is now on GitHub!

---

## Option B: Using SSH (Better for Repeated Pushes)

### Generate SSH Key (if you don't have one)

```bash
ssh-keygen -t ed25519 -C "your.email@example.com"
```

Press Enter for all prompts to use defaults.

### Add SSH Key to GitHub

```bash
# Copy the public key
cat ~/.ssh/id_ed25519.pub
```

1. Go to https://github.com/settings/keys
2. Click "New SSH key"
3. Paste the public key
4. Click "Add SSH key"

### Push Using SSH

```bash
cd /home/youssef-mansour@auc.egy/dist/phase1-steg-cluster

# Add remote using SSH
git remote add origin git@github.com:YOUR_USERNAME/phase1-steg-cluster.git

# Push
git push -u origin master
```

---

## ✅ Verify Push Succeeded

Go to your GitHub repository URL:
```
https://github.com/YOUR_USERNAME/phase1-steg-cluster
```

You should see:
- All files and folders
- Commit history showing "Initial commit"
- Code visible in the browser

---

## 🔄 STEP 3: Clone on Device 2 & Device 3

### On Device 2:

```bash
# Navigate to home directory
cd ~

# Clone the repository
git clone https://github.com/YOUR_USERNAME/phase1-steg-cluster.git

# Enter directory
cd phase1-steg-cluster

# Verify clone
ls -la  # Should see Cargo.toml, README.md, etc.
```

### On Device 3:

Same commands as Device 2:

```bash
cd ~
git clone https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
cd phase1-steg-cluster
ls -la
```

---

## 📝 Making Updates (if needed)

### On any device, after making changes:

```bash
# Check what changed
git status

# Stage changes
git add .

# Commit
git commit -m "Description of changes"

# Push
git push
```

### Pull changes on other devices:

```bash
git pull
```

---

## 🎬 Full Workflow Example

### Device 1 (Initial Push)

```bash
cd ~/phase1-steg-cluster

# Verify git status
git status

# Add remote
git remote add origin https://github.com/YOUR_USERNAME/phase1-steg-cluster.git

# Push
git push -u origin master
```

### Device 2 (Clone)

```bash
cd ~
git clone https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
cd phase1-steg-cluster

# Build
cargo build --release

# Configure for Device 2
nano config/cluster.yaml
# Edit: n2 IP to 10.0.0.12

# Run Device 2
export NODE_ID=n2
./bin/run-n2.sh
```

### Device 3 (Clone)

```bash
cd ~
git clone https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
cd phase1-steg-cluster

# Build
cargo build --release

# Configure for Device 3
nano config/cluster.yaml
# Edit: n3 IP to 10.0.0.13

# Run Device 3
export NODE_ID=n3
./bin/run-n3.sh
```

---

## 🚨 Troubleshooting

### Error: "Permission denied (publickey)"

**Solution:** You're using SSH but haven't added the key. Either:
- Add SSH key to GitHub (see above), or
- Use HTTPS instead

### Error: "fatal: remote origin already exists"

**Solution:** Remote is already configured. Check:
```bash
git remote -v
# To update:
git remote set-url origin https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
```

### Error: "authentication failed"

**Solution:** You're using wrong credentials:
- For HTTPS: Use personal access token, NOT your password
- For SSH: Make sure key is added to GitHub

### Changes on Device 1 not visible on Device 2

**Solution:** On Device 1, make sure you pushed:
```bash
git status  # Should show "nothing to commit"
```

On Device 2, pull the latest:
```bash
git pull
```

---

## 📋 Checklist

- [ ] GitHub account created
- [ ] Repository created at https://github.com/YOUR_USERNAME/phase1-steg-cluster
- [ ] Personal access token created (for HTTPS) or SSH key added
- [ ] Code pushed from Device 1: `git push -u origin master`
- [ ] GitHub repo shows all files
- [ ] Cloned on Device 2: `git clone https://...`
- [ ] Cloned on Device 3: `git clone https://...`
- [ ] All three devices have identical code
- [ ] Cluster running on all three devices
- [ ] All tests passing

---

## 🎉 You're Ready!

Your Phase-1 project is now:
✅ Version controlled with Git  
✅ Shared on GitHub  
✅ Cloned on all 3 devices  
✅ Ready for deployment  

**Next:** Follow DEVICE_SETUP.md for configuring each device! 🚀
