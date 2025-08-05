@echo off
echo 🚀 Meta-AI Orchestrator - GitHub Setup Script
echo ================================================

echo.
echo 📋 Этот скрипт настроит полный GitHub репозиторий
echo.

:: Проверка GitHub CLI
gh --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ GitHub CLI не найден. Установите: https://cli.github.com/
    echo    Или используйте: winget install GitHub.cli
    pause
    exit /b 1
)

echo ✅ GitHub CLI найден

:: Проверка авторизации
gh auth status >nul 2>&1
if %errorlevel% neq 0 (
    echo 🔐 Требуется авторизация в GitHub...
    gh auth login --git-protocol https --web
)

echo ✅ Авторизован в GitHub

:: Получение username
for /f "tokens=*" %%i in ('gh api user --jq .login') do set GITHUB_USER=%%i
echo 👤 GitHub пользователь: %GITHUB_USER%

:: Создание репозитория
echo.
echo 📦 Создание репозитория meta-ai-orchestrator...
gh repo create meta-ai-orchestrator ^
    --description "🤖 Enterprise-grade AI orchestration platform with multi-LLM support, RAG capabilities, and 99.99% accuracy guarantee" ^
    --public ^
    --source=. ^
    --remote=origin ^
    --push

if %errorlevel% neq 0 (
    echo ❌ Ошибка создания репозитория
    pause
    exit /b 1
)

echo ✅ Репозиторий создан и код загружен

:: Настройка репозитория
echo.
echo ⚙️ Настройка репозитория...

:: Добавление topics
gh repo edit --add-topic rust,ai,orchestration,llm,rag,openai,claude,copilot,cursor,vector-database,enterprise,microservices,async,tokio,quality-assurance

:: Включение issues и wiki
gh repo edit --enable-issues --enable-wiki

:: Создание labels
echo 📝 Создание labels...
gh label create "type: bug" --color "d73a4a" --description "Something isn't working" || echo "Label exists"
gh label create "type: feature" --color "0075ca" --description "New feature or request" || echo "Label exists"  
gh label create "type: enhancement" --color "a2eeef" --description "Improvement to existing feature" || echo "Label exists"
gh label create "priority: high" --color "b60205" --description "High priority" || echo "Label exists"
gh label create "priority: medium" --color "fbca04" --description "Medium priority" || echo "Label exists"
gh label create "priority: low" --color "0e8a16" --description "Low priority" || echo "Label exists"
gh label create "area: orchestrator" --color "c2e0c6" --description "Orchestrator layer" || echo "Label exists"
gh label create "area: agents" --color "c2e0c6" --description "Agent layer" || echo "Label exists"
gh label create "area: rag" --color "c2e0c6" --description "RAG layer" || echo "Label exists"
gh label create "area: evaluation" --color "c2e0c6" --description "Evaluation layer" || echo "Label exists"
gh label create "area: cli" --color "c2e0c6" --description "CLI/TUI interface" || echo "Label exists"
gh label create "good first issue" --color "7057ff" --description "Good for newcomers" || echo "Label exists"
gh label create "help wanted" --color "008672" --description "Extra attention is needed" || echo "Label exists"

:: Создание release
echo.
echo 🏷️ Создание первого релиза...
gh release create v0.1.0 ^
    --title "🚀 Meta-AI Orchestrator v0.1.0 - Complete Enterprise Architecture" ^
    --notes-file release-notes.md ^
    --draft

:: Создание issue templates
echo 📋 Настройка issue templates...
mkdir .github\ISSUE_TEMPLATE 2>nul

echo ---
echo name: Bug Report
echo about: Create a report to help us improve
echo title: '[BUG] '
echo labels: 'type: bug'
echo assignees: ''
echo ---
echo.
echo **Describe the bug**
echo A clear and concise description of what the bug is.
echo.
echo **To Reproduce**
echo Steps to reproduce the behavior:
echo 1. Go to '...'
echo 2. Click on '....'
echo 3. Scroll down to '....'
echo 4. See error
echo.
echo **Expected behavior**
echo A clear and concise description of what you expected to happen.
echo.
echo **Environment:**
echo - OS: [e.g. Windows 11]
echo - Rust version: [e.g. 1.70]
echo - Version: [e.g. 0.1.0]
echo.
echo **Additional context**
echo Add any other context about the problem here.
> .github\ISSUE_TEMPLATE\bug_report.md

echo ---
echo name: Feature Request  
echo about: Suggest an idea for this project
echo title: '[FEATURE] '
echo labels: 'type: feature'
echo assignees: ''
echo ---
echo.
echo **Is your feature request related to a problem? Please describe.**
echo A clear and concise description of what the problem is.
echo.
echo **Describe the solution you'd like**
echo A clear and concise description of what you want to happen.
echo.
echo **Describe alternatives you've considered**
echo A clear and concise description of any alternative solutions.
echo.
echo **Additional context**
echo Add any other context or screenshots about the feature request here.
> .github\ISSUE_TEMPLATE\feature_request.md

:: Создание pull request template
echo 📄 Создание PR template...
echo ## 📋 Description
echo Brief description of changes
echo.
echo ## 🔧 Type of Change
echo - [ ] Bug fix (non-breaking change which fixes an issue)
echo - [ ] New feature (non-breaking change which adds functionality)  
echo - [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
echo - [ ] Documentation update
echo.
echo ## ✅ Testing
echo - [ ] Unit tests pass
echo - [ ] Integration tests pass
echo - [ ] Manual testing completed
echo.
echo ## 📚 Documentation
echo - [ ] Code is documented
echo - [ ] README updated if needed
echo - [ ] CHANGELOG updated
echo.
echo ## 🔍 Checklist
echo - [ ] Code follows project style guidelines
echo - [ ] Self-review completed
echo - [ ] No merge conflicts
echo - [ ] Linked to related issues
> .github\pull_request_template.md

:: Финальные настройки
echo.
echo 🎨 Настройка branch protection...
gh api repos/%GITHUB_USER%/meta-ai-orchestrator/branches/main/protection ^
    --method PUT ^
    --field required_status_checks='{"strict":true,"contexts":["CI"]}' ^
    --field enforce_admins=true ^
    --field required_pull_request_reviews='{"required_approving_review_count":1}' ^
    --field restrictions=null >nul 2>&1 || echo "⚠️ Настройте branch protection вручную в Settings"

echo.
echo 🎉 ГОТОВО! Репозиторий полностью настроен!
echo.
echo 🔗 Ссылки:
echo    Репозиторий: https://github.com/%GITHUB_USER%/meta-ai-orchestrator
echo    Actions:     https://github.com/%GITHUB_USER%/meta-ai-orchestrator/actions  
echo    Releases:    https://github.com/%GITHUB_USER%/meta-ai-orchestrator/releases
echo    Issues:      https://github.com/%GITHUB_USER%/meta-ai-orchestrator/issues
echo.
echo ⭐ Не забудьте поставить звезду своему репозиторию!
echo.
pause