$repo_base=""
$mythic_code="Payload_Type/thanatos/mythic"
$agent_code="Payload_Type/thanatos/agent"


function Get-RepoBase {
    $repo_base_dir = Split-Path -Path "$PSScriptRoot/../.." -Resolve
    if (-Not (Test-Path -Path "$repo_base_dir/.git")) {
        Write-Host "Could not find git repository base"
        Exit 1
    }

    $repo_base = $repo_base_dir
}

function Check-SanitizerRequirements {
    Get-Command go -ErrorAction Stop | Out-Null
    Get-Command cargo -ErrorAction Stop | Out-Null
    cargo +nightly -V | Out-Null
    Get-Command Get-VSSetupInstance -ErrorAction Stop | Out-Null
    Get-VSSetupInstance | Select-VSSetupInstance -Require Microsoft.VisualStudio.Component.VC.ASAN | Out-Null
}

function Sanitizers {
    Write-Host "[*] Running sanitizer tests"

    $vspath = (Get-VSSetupInstance | Select-VSSetupInstance -Require Microsoft.VisualStudio.Component.VC.ASAN).InstallationPath
    Import-Module $vspath/Common7/Tools/Microsoft.VisualStudio.DevShell.dll
    Enter-VsDevShell -VsInstallPath $vspath -SkipAutomaticLocation | Out-Null

    if (-Not ($env:PATH -contains "$env:VCToolsInstallDir/bin/Hostx64/x64")) {
        $env:PATH="$env:PATH;$env:VCToolsInstallDir/bin/Hostx64/x64"
    }

    Push-Location -Path $agent_code
    $env:RUSTFLAGS = "-Zsanitizer=address"
    $env:CARGO_INCREMENTAL = "0"
    $cmd = "cargo +nightly test --color always -p ffiwrappers --all-features --target x86_64-pc-windows-msvc"
    Write-Host "current directory: $pwd"
    Write-Host "command: $cmd"
    Invoke-Expression -Command $cmd
    Pop-Location

    Remove-Item env:RUSTFLAGS
    Remove-Item env:CARGO_INCREMENTAL
}

Get-RepoBase
Check-SanitizerRequirements
Push-Location $repo_base
Sanitizers
Pop-Location
