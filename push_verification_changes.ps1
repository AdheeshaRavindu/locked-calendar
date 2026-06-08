# Push verification assets to origin/main
# Run this in PowerShell from the repo root: .\push_verification_changes.ps1

$files = @(
  'docs/privacy.html',
  'privacy.html',
  'icons/logo.svg',
  '.github/workflows/deploy-pages.yml',
  'verification-checklist.md',
  'assets/verification/demo_captions.md',
  'assets/verification/demo_script.md'
)

Write-Host "Staging files..."
foreach ($f in $files) {
  if (Test-Path $f) {
    git add $f
  } else {
    Write-Host "Warning: file not found: $f"
  }
}

$branch = 'main'

Write-Host "Committing changes..."
$commitMessage = "Add verification assets, privacy policy, SVG logo, and Pages deploy workflow"
try {
  git commit -m $commitMessage
} catch {
  Write-Host "No changes to commit or commit failed. Continuing to push."
}

Write-Host "Pushing to origin/$branch..."
try {
  git push origin $branch
  Write-Host "Push complete. Check Actions tab for the deploy workflow run."
} catch {
  Write-Host "Push failed. Please resolve remote issues (authentication, branch name) and try again."
}
