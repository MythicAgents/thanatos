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

function Check-TestRequirements {
    Get-Command go -ErrorAction Stop | Out-Null
    Get-Command cargo -ErrorAction Stop | Out-Null
}

function Test {
    Write-Host "[*] Running tests"

    Write-Host "[*] Mythic code"
    Push-Location -Path $mythic_code
    $cmd = "go test ./commands/..."
    Write-Host "current directory: $pwd"
    Write-Host "command: $cmd"
    Invoke-Expression -Command $cmd

    $cmd = "go test -run `"^TestPayloadMockBuild/`" ./builder"
    Write-Host "current directory: $pwd"
    Write-Host "command: $cmd"
    Invoke-Expression -Command $cmd
    Pop-Location

    Write-Host "[*] Agent code"
    Push-Location -Path $agent_code
    $cmd = "cargo test --color always --workspace --exclude genconfig --all-features"
    Write-Host "current directory: $pwd"
    Write-Host "command: $cmd"
    Invoke-Expression -Command $cmd
    Pop-Location
}

Get-RepoBase
Check-TestRequirements
Push-Location $repo_base
Test
Pop-Location
