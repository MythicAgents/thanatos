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

Get-RepoBase
Push-Location $repo_base
./.github/scripts/windows/checkformat.ps1
./.github/scripts/windows/lint.ps1
./.github/scripts/windows/test.ps1
./.github/scripts/windows/sanitizers.ps1

Pop-Location
