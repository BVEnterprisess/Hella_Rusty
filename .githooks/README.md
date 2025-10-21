# Git Hooks - Security Protection

This directory contains Git hooks to prevent accidental commits of secrets and credentials.

## 🔒 Pre-commit Hook

**Purpose**: Prevents commits that contain secrets, credentials, or sensitive data.

**What it checks**:
- `.env` files (blocks commits)
- Password patterns in code
- API keys and tokens
- Private keys
- Large files (warns about files >10MB)
- TODOs in security-critical files

## 🚀 Quick Setup

```bash
# From project root
cd /mnt/c/DevOps-Workspace/projects/Project-Chimera

# Run setup script
bash .githooks/setup-hooks.sh

# Or manually:
cp .githooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## ✅ Testing the Hook

```bash
# This will be BLOCKED:
echo "POSTGRES_PASSWORD=secret123" > test.txt
git add test.txt
git commit -m "test"  # ❌ Will fail

# This will be ALLOWED:
echo "POSTGRES_PASSWORD=\${POSTGRES_PASSWORD}" > config.txt
git add config.txt
git commit -m "test"  # ✅ Will succeed

# Bypass (NOT RECOMMENDED):
git commit --no-verify -m "bypass hooks"
```

## 🛠️ Maintenance

**Update hooks**:
```bash
# After pulling updates to hooks
bash .githooks/setup-hooks.sh
```

**Disable temporarily**:
```bash
# Remove hook
rm .git/hooks/pre-commit

# Re-enable later
bash .githooks/setup-hooks.sh
```

## 📚 More Information

See `docs/SECRETS_MANAGEMENT.md` for complete security guidelines.

---

**Created**: October 21, 2025 (Week 1, Day 1)  
**Status**: Active  
**Authority**: Security Emergency - Phase 0
