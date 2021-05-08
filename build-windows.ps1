Push-Location "$PSScriptRoot"
try {
    cargo build --release
} finally {
    Pop-Location
}
