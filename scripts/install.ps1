#Requires -Version 5.1
<#
.SYNOPSIS
    HTML2PDF Installer Script for Windows

.DESCRIPTION
    Downloads and installs the latest (or specified) version of HTML2PDF
    from GitHub releases.

.PARAMETER Version
    The version to install (default: latest)

.PARAMETER InstallDir
    The installation directory (default: $env:LOCALAPPDATA\Programs\html2pdf)

.PARAMETER AddToPath
    Add the installation directory to user PATH (default: true)

.PARAMETER Repo
    The GitHub repository to download from (default: yourusername/html2pdf-rs)

.PARAMETER NoVerifyChecksum
    Skip checksum verification

.EXAMPLE
    .\install.ps1

.EXAMPLE
    .\install.ps1 -Version "v0.1.0"

.EXAMPLE
    .\install.ps1 -InstallDir "C:\Tools" -AddToPath

.NOTES
    Run with -Verbose for detailed output
#>

[CmdletBinding()]
param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\html2pdf",
    [switch]$AddToPath = $true,
    [string]$Repo = "yourusername/html2pdf-rs",
    [switch]$NoVerifyChecksum
)

# Configuration
$BinaryName = "html2pdf.exe"
$ErrorActionPreference = "Stop"

# Helper functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Get-Architecture {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64" }
        "x86" { return "i686" }
        "ARM64" { return "aarch64" }
        default {
            Write-Error "Unsupported architecture: $arch"
            exit 1
        }
    }
}

function Get-LatestVersion {
    $apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
    
    try {
        $response = Invoke-RestMethod -Uri $apiUrl -Method Get
        return $response.tag_name
    }
    catch {
        Write-Error "Failed to get latest version: $_"
        exit 1
    }
}

function Get-DownloadUrl {
    param(
        [string]$Version,
        [string]$Arch
    )
    
    $target = "windows-$Arch"
    $packageName = "html2pdf-$Version-$target.zip"
    
    return "https://github.com/$Repo/releases/download/$Version/$packageName"
}

function Test-Command {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

function Expand-ZipArchive {
    param(
        [string]$Path,
        [string]$Destination
    )
    
    try {
        # Use .NET for better compatibility
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        [System.IO.Compression.ZipFile]::ExtractToDirectory($Path, $Destination)
    }
    catch {
        # Fallback to Expand-Archive (PowerShell 5.0+)
        Expand-Archive -Path $Path -DestinationPath $Destination -Force
    }
}

function Get-FileChecksum {
    param([string]$Path)
    
    $hash = Get-FileHash -Path $Path -Algorithm SHA256
    return $hash.Hash
}

function Add-ToPath {
    param([string]$Directory)
    
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    
    if ($userPath -like "*$Directory*") {
        Write-Info "Directory already in PATH"
        return
    }
    
    $newPath = "$userPath;$Directory"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    
    # Update current session
    $env:Path = "$env:Path;$Directory"
    
    Write-Info "Added to PATH: $Directory"
}

function Install-Html2Pdf {
    # Display header
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  HTML2PDF Installer" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
    
    # Detect architecture
    $arch = Get-Architecture
    Write-Info "Detected Architecture: $arch"
    
    # Get version
    if ($Version -eq "latest") {
        Write-Info "Fetching latest version..."
        $Version = Get-LatestVersion
    }
    
    Write-Info "Version: $Version"
    
    # Create temp directory
    $tempDir = Join-Path $env:TEMP "html2pdf-install-$(Get-Random)"
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
    
    try {
        # Get download URL
        $downloadUrl = Get-DownloadUrl -Version $Version -Arch $arch
        Write-Verbose "Download URL: $downloadUrl"
        
        # Download archive
        $archivePath = Join-Path $tempDir "html2pdf.zip"
        Write-Info "Downloading HTML2PDF..."
        
        try {
            Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -UseBasicParsing
        }
        catch {
            Write-Error "Download failed"
            Write-Error "URL: $downloadUrl"
            Write-Error "This might be because:"
            Write-Error "  - The version doesn't exist"
            Write-Error "  - Your platform is not supported"
            Write-Error "  - Network issues"
            exit 1
        }
        
        Write-Info "Downloaded to: $archivePath"
        
        # Verify checksum
        if (-not $NoVerifyChecksum) {
            $checksumUrl = "$downloadUrl.sha256"
            $checksumPath = "$archivePath.sha256"
            
            try {
                Invoke-WebRequest -Uri $checksumUrl -OutFile $checksumPath -UseBasicParsing
                $expectedChecksum = Get-Content $checksumPath
                $actualChecksum = Get-FileChecksum -Path $archivePath
                
                if ($expectedChecksum -ne $actualChecksum) {
                    Write-Error "Checksum verification failed!"
                    Write-Error "Expected: $expectedChecksum"
                    Write-Error "Actual:   $actualChecksum"
                    exit 1
                }
                
                Write-Info "Checksum verified"
            }
            catch {
                Write-Warn "Could not verify checksum: $_"
            }
        }
        
        # Extract archive
        $extractDir = Join-Path $tempDir "extract"
        Write-Info "Extracting archive..."
        Expand-ZipArchive -Path $archivePath -Destination $extractDir
        
        # Find binary
        $binaryPath = Get-ChildItem -Path $extractDir -Filter $BinaryName -Recurse | Select-Object -First 1
        
        if (-not $binaryPath) {
            Write-Error "Binary not found in archive"
            exit 1
        }
        
        Write-Verbose "Found binary at: $($binaryPath.FullName)"
        
        # Create installation directory
        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        
        # Install binary
        $destPath = Join-Path $InstallDir $BinaryName
        Write-Info "Installing to: $destPath"
        Copy-Item -Path $binaryPath.FullName -Destination $destPath -Force
        
        # Install documentation
        $docFiles = @("README.md", "LICENSE", "CHANGELOG.md")
        foreach ($file in $docFiles) {
            $srcFile = Join-Path $extractDir $file
            if (Test-Path $srcFile) {
                Copy-Item -Path $srcFile -Destination $InstallDir -Force -ErrorAction SilentlyContinue
            }
        }
        
        # Add to PATH
        if ($AddToPath) {
            Write-Info "Adding to PATH..."
            Add-ToPath -Directory $InstallDir
        }
        
        # Verify installation
        Write-Info "Verifying installation..."
        $installedVersion = & $destPath --version 2>$null
        
        if ($LASTEXITCODE -ne 0) {
            Write-Warn "Installation verification returned non-zero exit code"
        }
        else {
            Write-Info "Installed version: $installedVersion"
        }
        
        # Success message
        Write-Host ""
        Write-Host "========================================" -ForegroundColor Green
        Write-Host "  Installation Successful!" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "HTML2PDF has been installed to: $InstallDir"
        
        if ($AddToPath) {
            Write-Host ""
            Write-Host "Please restart your terminal to use 'html2pdf' command"
        }
        else {
            Write-Host ""
            Write-Host "To use html2pdf, either:"
            Write-Host "  1. Add $InstallDir to your PATH, or"
            Write-Host "  2. Use the full path: $destPath"
        }
        
        Write-Host ""
        Write-Host "Run 'html2pdf --help' to get started"
    }
    finally {
        # Cleanup
        if (Test-Path $tempDir) {
            Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

# Main execution
try {
    Install-Html2Pdf
}
catch {
    Write-Error "Installation failed: $_"
    Write-Error $_.ScriptStackTrace
    exit 1
}
