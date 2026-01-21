# BACKit-onStellar Monorepo Setup Guide

## Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** v18.18.0 or higher (check with `node --version`)
- **pnpm** v8.0.0 or higher (install with `npm install -g pnpm`)
- **Git** (for version control)

## Setup Instructions

### Option 1: Using the Setup Script (Windows)

1. Open Command Prompt (cmd.exe) as Administrator
2. Navigate to the project directory:
   ```cmd
   cd c:\Users\g-ekoh\Desktop\BACKit-onStellar
   ```
3. Run the setup script:
   ```cmd
   setup.bat
   ```

### Option 2: Manual Setup

If you encounter PowerShell execution policy issues, follow these steps:

1. **Open Command Prompt (cmd.exe)** instead of PowerShell

2. **Navigate to the project directory:**
   ```cmd
   cd c:\Users\g-ekoh\Desktop\BACKit-onStellar
   ```

3. **Install dependencies:**
   ```cmd
   pnpm install
   ```

4. **Verify the installation:**
   ```cmd
   pnpm list
   ```

5. **Test the dev command (optional):**
   ```cmd
   pnpm run dev
   ```
   *Note: This will show "no dev script" errors since packages are not yet configured, which is expected.*

### Option 3: Fix PowerShell Execution Policy

If you prefer to use PowerShell, you can temporarily change the execution policy:

1. **Open PowerShell as Administrator**

2. **Check current policy:**
   ```powershell
   Get-ExecutionPolicy
   ```

3. **Set policy to RemoteSigned (recommended) or Unrestricted:**
   ```powershell
   Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

4. **Then run:**
   ```powershell
   cd c:\Users\g-ekoh\Desktop\BACKit-onStellar
   pnpm install
   ```

## Verification Steps

After installation, verify the setup:

1. **Check workspace structure:**
   ```cmd
   dir packages
   ```
   You should see: `frontend`, `backend`, and `contracts` directories

2. **Verify pnpm workspace:**
   ```cmd
   type pnpm-workspace.yaml
   ```

3. **Check Turborepo config:**
   ```cmd
   type turbo.json
   ```

4. **List installed packages:**
   ```cmd
   pnpm list --depth=0
   ```

## Expected Output

When you run `pnpm install`, you should see:

```
Lockfile is up to date, resolution step is skipped
Packages: +3
+++
Progress: resolved 3, reused 0, downloaded 3, added 3, done

dependencies:
+ prettier 3.2.5
+ turbo 1.13.0
+ typescript 5.3.3

Done in 15s
```

## Troubleshooting

### Issue: "pnpm: command not found"

**Solution:** Install pnpm globally:
```cmd
npm install -g pnpm
```

### Issue: "PowerShell execution policy error"

**Solution:** Use Command Prompt (cmd.exe) instead, or follow Option 3 above to fix the policy.

### Issue: "Node version incompatible"

**Solution:** 
1. Install Node Version Manager (nvm):
   - Download from: https://github.com/coreybutler/nvm-windows/releases
2. Install the correct version:
   ```cmd
   nvm install 18.18.0
   nvm use 18.18.0
   ```

### Issue: "Permission denied" errors

**Solution:** Run Command Prompt or PowerShell as Administrator

## Directory Structure

After successful setup, your structure should look like:

```
BACKit-onStellar/
├── packages/
│   ├── frontend/
│   │   ├── package.json
│   │   └── README.md
│   ├── backend/
│   │   ├── package.json
│   │   └── README.md
│   └── contracts/
│       ├── package.json
│       └── README.md
├── .gitignore
├── .nvmrc
├── package.json
├── pnpm-workspace.yaml
├── turbo.json
├── setup.bat
├── SETUP.md (this file)
├── ARCHITECTURE.md
└── README.md
```

## Next Steps

1. **Frontend Setup:**
   ```cmd
   cd packages\frontend
   REM Follow the README.md instructions to set up Next.js
   ```

2. **Backend Setup:**
   ```cmd
   cd packages\backend
   REM Follow the README.md instructions to set up NestJS
   ```

3. **Contracts Setup:**
   ```cmd
   cd packages\contracts
   REM Install Rust and Soroban CLI (see contracts README.md)
   ```

## Running the Monorepo

Once all packages are configured:

```cmd
# Development mode (all packages)
pnpm dev

# Build all packages
pnpm build

# Run tests across all packages
pnpm test

# Lint all packages
pnpm lint
```

## Additional Resources

- **pnpm Documentation:** https://pnpm.io/
- **Turborepo Documentation:** https://turbo.build/repo/docs
- **Stellar/Soroban Documentation:** https://developers.stellar.org/docs
- **Next.js Documentation:** https://nextjs.org/docs
- **NestJS Documentation:** https://docs.nestjs.com

## Support

If you encounter any issues not covered here, please:

1. Check the issue tracker on GitHub
2. Review the ARCHITECTURE.md file for detailed system design
3. Consult package-specific README files

---

**Setup completed successfully?** You're ready to start developing! 🚀
