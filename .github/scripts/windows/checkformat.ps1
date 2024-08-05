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

function Check-FormatRequirements {
    Get-Command gofmt -ErrorAction Stop | Out-Null
    Get-Command cargo -ErrorAction Stop | Out-Null
}

function CheckFormat {
    Write-Host "[*] Running code format checks"

    Write-Host "[*] Agent code"
    Push-Location -Path $agent_code
    $cmd = "cargo fmt --all -- --color always --check"
    Write-Host "current directory: $pwd"
    Write-Host "command: $cmd"
    Invoke-Expression -Command $cmd
    Pop-Location
}

Get-RepoBase
Check-FormatRequirements
Push-Location $repo_base
CheckFormat
Pop-Location
