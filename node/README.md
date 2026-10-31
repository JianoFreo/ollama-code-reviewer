# .env
```.env
# AI Configuration -- locally 
AI_PROVIDER=ollama
AI_BASE_URL=http://localhost:11434
AI_MODEL=qwen2.5-coder:7b
AI_API_KEY=Leave it empty. Local Ollama doesn’t need a key.

# AI Configuration -- cloud based
AI_PROVIDER=openai
AI_BASE_URL=https://api.openai.com/v1
AI_MODEL=gpt-6
AI_API_KEY=your_openai_api_key

# GitHub Configuration
GITHUB_API_URL=https://api.github.com #Leave it as is. Only change it if you use GitHub Enterprise.
GITHUB_TOKEN=
GITHUB_OWNER=JianoFreo #The first part of the repo URL: github.com/owner/repo
GITHUB_REPO=# The second part of the repo URL: github.com/owner/repo

Paste it in: GITHUB_TOKEN=github_pat_...
# Application
NODE_ENV=development
PORT=3000
```
# Generate the token
   - Go to github.com → click your profile picture → Settings.
   - Scroll down to Developer settings → Personal access tokens → Fine-grained tokens.
   - Click Generate new token and give it a name.
   - Under Repository access, choose Only select repositories and pick the repo from above.
   - Under Permissions → Repository permissions, set Pull requests to Read and write. (GitHub adds Metadata: Read automatically.)
   - Click Generate token and copy it right away, because GitHub only shows it once.