$repo_base=""
$mythic_code="Payload_Type/thanatos"
$agent_code="Payload_Type/thanatos/agent"


function Get-RepoBase {
    $repo_base_dir = Split-Path -Path "$PSScriptRoot/../.." -Resolve
    if (-Not (Test-Path -Path "$repo_base_dir/.git")) {
        Write-Host "Could not find git repository base"
        Exit 1
    }

    $repo_base = $repo_base_dir
}

function Check-LintRequirements {
    Get-Command golangci-lint -ErrorAction Stop | Out-Null
    cargo clippy -V | Out-Null
}

function Lint {
    Write-Host "[*] Running lint checks"

    Write-Host "[*] Mythic code"
    Push-Location -Path $mythic_code
    $cmd = "golangci-lint run"
    Write-Host "current directory: $pwd"
    Write-Host "command: $cmd"
    Invoke-Expression -Command $cmd
    Pop-Location

    Write-Host "[*] Agent code"
    Push-Location -Path $agent_code
    $cmd = "cargo clippy --workspace --color always --all-features --all-targets -- -D warnings"
    Write-Host "current directory: $pwd"
    Write-Host "command: $cmd"
    Invoke-Expression -Command $cmd
    Pop-Location
}

Get-RepoBase
Check-LintRequirements
Push-Location $repo_base
Lint
Pop-Location
