# KeyFlow Windows Service Installer
# Usage: .\install.ps1 [-Uninstall] [-Start] [-Stop] [-Status]

param(
    [switch]$Uninstall,
    [switch]$Start,
    [switch]$Stop,
    [switch]$Status,
    [string]$InstallPath = "$env:LOCALAPPDATA\KeyFlow"
)

$ServiceName = "KeyFlow"
$DisplayName = "KeyFlow - Non-paste password input assistant"
$Description = "Simulates keystrokes to bypass paste-disabled password fields"

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Install-KeyFlow {
    Write-Host "Installing KeyFlow..." -ForegroundColor Cyan

    # Create install directory
    if (-not (Test-Path $InstallPath)) {
        New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
        Write-Host "  Created directory: $InstallPath"
    }

    # Copy binary
    $binaryPath = Join-Path $PSScriptRoot "keyflow.exe"
    if (Test-Path $binaryPath) {
        Copy-Item $binaryPath "$InstallPath\keyflow.exe" -Force
        Write-Host "  Installed binary: $InstallPath\keyflow.exe"
    } else {
        Write-Host "  Error: keyflow.exe not found in script directory" -ForegroundColor Red
        exit 1
    }

    # Copy config example
    $configExample = Join-Path $PSScriptRoot "keyflow.toml.example"
    if (Test-Path $configExample) {
        $configPath = "$InstallPath\keyflow.toml"
        if (-not (Test-Path $configPath)) {
            Copy-Item $configExample $configPath
            Write-Host "  Created config: $configPath"
        } else {
            Write-Host "  Config already exists: $configPath"
        }
    }

    # Add to PATH if not already there
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$InstallPath*") {
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallPath", "User")
        $env:Path = "$env:Path;$InstallPath"
        Write-Host "  Added to PATH: $InstallPath"
    }

    # Register as Windows service (requires NSSM or sc.exe)
    # Using sc.exe for native Windows service support
    $keyflowPath = "$InstallPath\keyflow.exe"

    # Remove existing service if present
    $existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($existingService) {
        Write-Host "  Removing existing service..."
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        sc.exe delete $ServiceName | Out-Null
        Start-Sleep -Seconds 2
    }

    # Create service
    # Note: keyflow runs as a foreground process, so we use sc.exe with type=own
    sc.exe create $ServiceName binPath= "`"$keyflowPath`" run" start= demand DisplayName= $DisplayName | Out-Null
    sc.exe description $ServiceName $Description | Out-Null

    Write-Host ""
    Write-Host "KeyFlow installed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:"
    Write-Host "  1. Edit config: $InstallPath\keyflow.toml"
    Write-Host "  2. Start service: .\install.ps1 -Start"
    Write-Host "  3. Or run directly: keyflow run"
}

function Uninstall-KeyFlow {
    Write-Host "Uninstalling KeyFlow..." -ForegroundColor Cyan

    # Stop and remove service
    $existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($existingService) {
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        sc.exe delete $ServiceName | Out-Null
        Write-Host "  Removed service: $ServiceName"
    }

    # Remove from PATH
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -like "*$InstallPath*") {
        $newPath = $currentPath -replace [regex]::Escape(";$InstallPath"), ""
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Host "  Removed from PATH: $InstallPath"
    }

    # Remove installation directory
    if (Test-Path $InstallPath) {
        Remove-Item $InstallPath -Recurse -Force
        Write-Host "  Removed directory: $InstallPath"
    }

    Write-Host ""
    Write-Host "KeyFlow uninstalled." -ForegroundColor Green
    Write-Host "  Config files may be preserved in: $env:APPDATA\keyflow"
}

function Start-KeyFlow {
    $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($service) {
        Start-Service -Name $ServiceName
        Write-Host "KeyFlow service started." -ForegroundColor Green
    } else {
        Write-Host "KeyFlow service not found. Run .\install.ps1 first." -ForegroundColor Red
    }
}

function Stop-KeyFlow {
    $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($service) {
        Stop-Service -Name $ServiceName -Force
        Write-Host "KeyFlow service stopped." -ForegroundColor Green
    } else {
        Write-Host "KeyFlow service not found." -ForegroundColor Red
    }
}

function Show-Status {
    $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($service) {
        Write-Host "KeyFlow Service Status:" -ForegroundColor Cyan
        Write-Host "  Name: $($service.Name)"
        Write-Host "  Status: $($service.Status)"
        Write-Host "  StartType: $($service.StartType)"
    } else {
        Write-Host "KeyFlow service not installed." -ForegroundColor Yellow
    }

    # Check if keyflow is running
    $process = Get-Process -Name "keyflow" -ErrorAction SilentlyContinue
    if ($process) {
        Write-Host ""
        Write-Host "KeyFlow Process:" -ForegroundColor Cyan
        Write-Host "  PID: $($process.Id)"
        Write-Host "  Memory: $([math]::Round($process.WorkingSet64/1MB, 2)) MB"
    }
}

# Main logic
if ($Uninstall) {
    Uninstall-KeyFlow
} elseif ($Start) {
    Start-KeyFlow
} elseif ($Stop) {
    Stop-KeyFlow
} elseif ($Status) {
    Show-Status
} else {
    Install-KeyFlow
}
