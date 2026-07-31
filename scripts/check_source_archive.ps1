[CmdletBinding()]
param(
    [string]$Treeish = "HEAD",
    [switch]$Worktree,
    [switch]$UseWorktreeAttributes,
    [switch]$SkipValidation
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$workspaceRoot = [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot ".."))
$runId = [guid]::NewGuid().ToString("N")
$workRoot = [System.IO.Path]::GetFullPath((Join-Path $workspaceRoot ("plc-slmp-source-archive-" + $runId)))
$archivePath = Join-Path $workRoot "source.zip"
$extractRoot = Join-Path $workRoot "extracted"
$temporaryIndexPath = Join-Path $workspaceRoot ("plc-slmp-source-archive-" + $runId + ".index")
$workspacePrefix = $workspaceRoot.TrimEnd([System.IO.Path]::DirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar
if (-not $workRoot.StartsWith($workspacePrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Source-archive work directory is outside the workspace: $workRoot"
}

try {
    [void](New-Item -ItemType Directory -Path $workRoot -Force)
    if ($Worktree) {
        $previousIndexFile = $env:GIT_INDEX_FILE
        try {
            $env:GIT_INDEX_FILE = $temporaryIndexPath
            & git -C $repositoryRoot read-tree HEAD
            if ($LASTEXITCODE -ne 0) { throw "Cannot initialize the temporary current-worktree index." }
            & git -C $repositoryRoot add -A -- .
            if ($LASTEXITCODE -ne 0) { throw "Cannot stage the complete current worktree in the temporary index." }
            $archiveTreeish = (& git -C $repositoryRoot write-tree).Trim()
            if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($archiveTreeish)) {
                throw "Cannot create the synthetic current-worktree tree."
            }
        }
        finally {
            if ($null -eq $previousIndexFile) {
                Remove-Item Env:GIT_INDEX_FILE -ErrorAction SilentlyContinue
            }
            else {
                $env:GIT_INDEX_FILE = $previousIndexFile
            }
        }
        $archiveLabel = "WORKTREE"
    }
    else {
        & git -C $repositoryRoot rev-parse --verify "$Treeish`^{tree}" *> $null
        if ($LASTEXITCODE -ne 0) { throw "Cannot resolve treeish '$Treeish'." }
        $archiveTreeish = $Treeish
        $archiveLabel = $Treeish
    }

    $archiveArguments = @("archive", "--format=zip", "--output=$archivePath")
    if ($UseWorktreeAttributes -or $Worktree) { $archiveArguments += "--worktree-attributes" }
    $archiveArguments += $archiveTreeish
    & git -C $repositoryRoot @archiveArguments
    if ($LASTEXITCODE -ne 0) { throw "git archive failed for '$archiveLabel'." }
    $trackedFiles = @(& git -C $repositoryRoot ls-tree -r --name-only $archiveTreeish |
        ForEach-Object { $_.Replace("\", "/") } |
        Sort-Object -Unique)
    if ($LASTEXITCODE -ne 0) { throw "Cannot enumerate tracked files for '$archiveLabel'." }

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [System.IO.Compression.ZipFile]::OpenRead($archivePath)
    try {
        $archiveFiles = @($archive.Entries |
            Where-Object { -not $_.FullName.EndsWith("/") } |
            ForEach-Object { $_.FullName.Replace("\", "/") } |
            Sort-Object -Unique)
    }
    finally {
        $archive.Dispose()
    }

    $requiredRootFiles = @("CHANGELOG.md", "LICENSE", "README.md", "run_ci.bat")
    $missingRootFiles = @($requiredRootFiles | Where-Object { $_ -notin $archiveFiles })
    if ($missingRootFiles.Count -ne 0) {
        throw "Source archive is missing required root files: $($missingRootFiles -join ', ')"
    }

    $manifestCandidates = @("package.json", "pyproject.toml", "Cargo.toml", "library.json", "library.properties", "CMakeLists.txt") +
        @($trackedFiles | Where-Object { $_ -match '\.(sln|csproj)$' })
    if (@($manifestCandidates | Where-Object { $_ -in $archiveFiles }).Count -eq 0) {
        throw "Source archive contains no recognized build manifest."
    }

    $guideRoots = @("docsrc/user", "docs")
    foreach ($guide in @("GETTING_STARTED.md", "USAGE_GUIDE.md", "PROFILES.md", "GOTCHAS.md", "API_REFERENCE.md")) {
        if (@($guideRoots | ForEach-Object { "$_/$guide" } | Where-Object { $_ -in $archiveFiles }).Count -eq 0) {
            throw "Source archive is missing standard user guide '$guide'."
        }
    }

    $requiredTracked = @($trackedFiles | Where-Object {
        $_ -match '^(test|tests|\.github|docsrc/maintainer|internal_docs|scripts|tools)/' -or
        $_ -in @("AGENTS.md", "TODO.md", "release_check.bat", "run_ci.bat")
    })
    $missingTracked = @($requiredTracked | Where-Object { $_ -notin $archiveFiles })
    if ($missingTracked.Count -ne 0) {
        throw "Source archive omits tracked validation or maintainer material: $($missingTracked -join ', ')"
    }
    if (@($archiveFiles | Where-Object { $_ -match '^(test|tests)/' }).Count -eq 0) {
        throw "Source archive contains no repository tests."
    }

    $forbidden = @($archiveFiles | Where-Object {
        $_ -match '^(build|build_win|release-artifacts)/'
    })
    if ($forbidden.Count -ne 0) {
        throw "Source archive contains generated or release-output files: $($forbidden -join ', ')"
    }

    if (-not $SkipValidation) {
        Expand-Archive -LiteralPath $archivePath -DestinationPath $extractRoot
        Push-Location $extractRoot
        try {
            if ($env:OS -eq "Windows_NT") {
                & cmd.exe /d /c run_ci.bat
                if ($LASTEXITCODE -ne 0) { throw "run_ci.bat failed from the extracted source archive." }
            }
            else {
                & cargo fmt --all --check
                if ($LASTEXITCODE -ne 0) { throw "cargo fmt failed from the extracted source archive." }
                & cargo clippy --all-targets --features cli -- -D warnings
                if ($LASTEXITCODE -ne 0) { throw "cargo clippy failed from the extracted source archive." }
                & cargo test
                if ($LASTEXITCODE -ne 0) { throw "cargo test failed from the extracted source archive." }
                & cargo check -p slmp-node
                if ($LASTEXITCODE -ne 0) { throw "slmp-node check failed from the extracted source archive." }
            }
        }
        finally {
            Pop-Location
        }
    }

    Write-Host "[OK] Source archive contract passed: treeish=$archiveLabel files=$($archiveFiles.Count) validation=$(-not $SkipValidation)"
}
finally {
    if (Test-Path -LiteralPath $workRoot) {
        Remove-Item -LiteralPath $workRoot -Recurse -Force
    }
    Remove-Item -LiteralPath $temporaryIndexPath -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath "$temporaryIndexPath.lock" -Force -ErrorAction SilentlyContinue
}
