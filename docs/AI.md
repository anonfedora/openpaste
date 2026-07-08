# OpenPaste AI Features

## Overview

OpenPaste includes optional AI features to enhance clipboard management. AI features are opt-in and designed to respect user privacy. This document describes the AI architecture, features, and implementation approach.

## AI Philosophy

**Principles:**
- Privacy-first: Data processed locally when possible
- Opt-in: AI features require explicit user consent
- Transparent: Users know when AI is used
- Optional: Core functionality works without AI
- Secure: AI providers handled securely

## AI Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    AI Module                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Local AI    │  │  Cloud AI   │  │  Hybrid AI   │      │
│  │  (Optional)  │  │  (Optional)  │  │  (Optional)  │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
└─────────┼──────────────────┼──────────────────┼──────────────┘
          │                  │                  │
    ┌─────▼─────┐    ┌──────▼──────┐    ┌──────▼──────┐
    │  Content  │    │  Search    │    │  Analysis  │
    │Categorizer│    │Enhancement │    │  Engine    │
    └───────────┘    └─────────────┘    └─────────────┘
```

## AI Features

### Content Categorization

**Purpose:** Automatically categorize clipboard items

**Implementation:**
- Local: Simple rule-based categorization
- Cloud: ML model for advanced categorization

**Categories:**
- Code (by language)
- URLs (by type)
- Email addresses
- Phone numbers
- Addresses
- Dates
- Financial data
- Personal information

**Privacy:** Local processing by default

### Smart Search

**Purpose:** Improve search relevance with AI

**Features:**
- Semantic search
- Query expansion
- Result reranking
- Natural language queries

**Implementation:**
- Local: Embedding-based search (future)
- Cloud: OpenAI API or similar

**Privacy:** User choice between local and cloud

### Content Analysis

**Purpose:** Analyze clipboard content for insights

**Features:**
- Text summarization
- Key extraction
- Sentiment analysis
- Language detection
- Code analysis

**Implementation:**
- Cloud: OpenAI API, Anthropic API, or local LLM

**Privacy:** Explicit consent required

### Smart Suggestions

**Purpose:** Suggest actions based on content

**Features:**
- Suggest tags based on content
- Suggest collections
- Suggest related items
- Suggest formatting

**Implementation:**
- Local: Rule-based suggestions
- Cloud: ML-based suggestions

**Privacy:** Local by default

## AI Providers

### Local AI

**Advantages:**
- Privacy: Data never leaves device
- Speed: No network latency
- Cost: No API fees
- Offline: Works without internet

**Disadvantages:**
- Limited capabilities
- Resource intensive
- Slower for complex tasks
- Requires local hardware

**Technologies:**
- Candle (Rust ML framework)
- ONNX Runtime
- Local LLMs (Llama, Mistral)

### Cloud AI

**Advantages:**
- Advanced capabilities
- Faster for complex tasks
- Regular updates
- Less resource intensive

**Disadvantages:**
- Privacy: Data sent to cloud
- Cost: API fees
- Requires internet
- Latency

**Providers:**
- OpenAI (GPT-4, GPT-3.5)
- Anthropic (Claude)
- Google (Gemini)
- LocalAI (self-hosted)

### Hybrid AI

**Approach:** Use local AI for simple tasks, cloud AI for complex tasks

**Decision Logic:**
- Local: Text categorization, simple suggestions
- Cloud: Summarization, complex analysis
- Fallback: Cloud if local fails

## Implementation

### AI Module Structure

```rust
// clipboard-ai crate

pub mod local;
pub mod cloud;
pub mod hybrid;

pub trait AIProvider {
    fn categorize(&self, content: &str) -> Result<Vec<Category>, AIError>;
    fn summarize(&self, content: &str) -> Result<String, AIError>;
    fn extract_keys(&self, content: &str) -> Result<Vec<String>, AIError>;
}

pub struct LocalAI {
    // Local ML models
}

pub struct CloudAI {
    api_key: String,
    provider: AIProviderType,
}

pub struct HybridAI {
    local: LocalAI,
    cloud: Option<CloudAI>,
}
```

### Content Categorization

**Local Implementation:**
```rust
pub fn categorize_local(content: &str) -> Vec<Category> {
    let mut categories = Vec::new();
    
    // Rule-based categorization
    if is_code(content) {
        categories.push(Category::Code);
    }
    
    if is_url(content) {
        categories.push(Category::URL);
    }
    
    if is_email(content) {
        categories.push(Category::Email);
    }
    
    categories
}
```

**Cloud Implementation:**
```rust
pub async fn categorize_cloud(content: &str, api_key: &str) -> Result<Vec<Category>, AIError> {
    let client = OpenAIClient::new(api_key);
    let response = client
        .chat()
        .model("gpt-4")
        .messages(vec![
            Message::system("Categorize the following clipboard content"),
            Message::user(content),
        ])
        .await?;
    
    parse_categories(&response)
}
```

### Smart Search

**Semantic Search:**
```rust
pub async fn semantic_search(query: &str, items: &[ClipboardItem]) -> Vec<ClipboardItem> {
    // Generate embeddings
    let query_embedding = generate_embedding(query).await?;
    
    let mut scored_items = Vec::new();
    for item in items {
        let item_embedding = get_or_generate_embedding(item).await?;
        let similarity = cosine_similarity(&query_embedding, &item_embedding);
        scored_items.push((item.clone(), similarity));
    }
    
    scored_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored_items.into_iter().map(|(item, _)| item).collect()
}
```

### Content Summarization

**Implementation:**
```rust
pub async fn summarize(content: &str, api_key: &str) -> Result<String, AIError> {
    let client = OpenAIClient::new(api_key);
    let response = client
        .chat()
        .model("gpt-4")
        .messages(vec![
            Message::system("Summarize the following text concisely"),
            Message::user(content),
        ])
        .max_tokens(150)
        .await?;
    
    Ok(response.content)
}
```

## Configuration

### AI Settings

**User Config:**
```json
{
  "ai": {
    "enabled": false,
    "provider": "local",
    "categorization": {
      "enabled": true,
      "auto_tag": true
    },
    "search": {
      "semantic_search": false,
      "query_expansion": false
    },
    "analysis": {
      "summarization": false,
      "key_extraction": false
    },
    "cloud": {
      "provider": "openai",
      "api_key": "",
      "model": "gpt-4"
    }
  }
}
```

### Provider Configuration

**OpenAI:**
```json
{
  "provider": "openai",
  "api_key": "sk-...",
  "model": "gpt-4",
  "endpoint": "https://api.openai.com/v1"
}
```

**Anthropic:**
```json
{
  "provider": "anthropic",
  "api_key": "sk-ant-...",
  "model": "claude-3-opus-20240229",
  "endpoint": "https://api.anthropic.com/v1"
}
```

**LocalAI:**
```json
{
  "provider": "local",
  "model_path": "/path/to/model.gguf",
  "endpoint": "http://localhost:11434"
}
```

## Privacy

### Data Handling

**Local AI:**
- Data never leaves device
- No external API calls
- Full privacy

**Cloud AI:**
- Data sent to provider
- User consent required
- Data not stored by provider (depends on provider policy)
- Optional data anonymization

### Consent

**Explicit Consent:**
- AI features opt-in
- Each feature requires consent
- Consent can be revoked
- Clear indication when AI is used

**Data Minimization:**
- Only send necessary data
- Anonymize when possible
- Remove sensitive data

### Transparency

**UI Indicators:**
- Show when AI is processing
- Show which provider is used
- Show data being sent (preview)

**Logging:**
- Log AI usage
- Log provider used
- Log data sent (hash only)

## Performance

### Performance Targets

**Local AI:**
- Categorization: < 100ms
- Simple suggestions: < 50ms

**Cloud AI:**
- Categorization: < 500ms
- Summarization: < 2s
- Semantic search: < 1s

### Optimization

**Caching:**
- Cache embeddings
- Cache categorizations
- Cache summaries

**Batching:**
- Batch API calls
- Process multiple items together

**Fallback:**
- Fallback to local if cloud fails
- Fallback to rule-based if AI fails

## Cost Management

### Cloud AI Costs

**Cost Estimation:**
- OpenAI GPT-4: ~$0.03/1K tokens
- OpenAI GPT-3.5: ~$0.002/1K tokens
- Anthropic Claude: ~$0.015/1K tokens

**Cost Controls:**
- Monthly budget limit
- Per-feature limits
- User notifications

### Usage Tracking

**Metrics:**
- API calls per month
- Tokens used per month
- Cost per month
- Cost per feature

**Alerts:**
- Alert when approaching budget
- Alert when budget exceeded
- Suggest cost-saving options

## Security

### API Key Management

**Storage:**
- Encrypted storage
- System keychain (optional)
- Environment variable (optional)

**Rotation:**
- Regular rotation recommended
- User can rotate key
- Key validation

### Data Security

**Encryption:**
- Encrypt data in transit (HTTPS)
- Encrypt data at rest (if stored)

**Validation:**
- Validate API responses
- Sanitize AI outputs
- Prevent injection attacks

## Testing

### Unit Tests

**Test AI Logic:**
```rust
#[test]
fn test_categorize_code() {
    let content = "fn main() { println!(\"Hello\"); }";
    let categories = categorize_local(content);
    assert!(categories.contains(&Category::Code));
}
```

### Integration Tests

**Test Cloud AI:**
```rust
#[tokio::test]
#[ignore] // Requires API key
async fn test_cloud_categorization() {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap();
    let categories = categorize_cloud("test content", &api_key).await.unwrap();
    assert!(!categories.is_empty());
}
```

### Mock Tests

**Mock Cloud AI:**
```rust
struct MockAIProvider;

impl AIProvider for MockAIProvider {
    fn categorize(&self, content: &str) -> Result<Vec<Category>, AIError> {
        Ok(vec![Category::Text])
    }
}
```

## Future Enhancements

### Advanced Features

**Code Analysis:**
- Code quality analysis
- Bug detection
- Security vulnerability detection
- Code refactoring suggestions

**Image Analysis:**
- OCR for text extraction
- Image categorization
- Duplicate image detection
- Image quality analysis

**Audio/Video:**
- Transcription
- Content analysis
- Metadata extraction

### Better Local AI

**Smaller Models:**
- Quantized models
- Distilled models
- Specialized models

**Faster Inference:**
- GPU acceleration
- Model optimization
- Caching strategies

### Custom Models

**User Training:**
- Fine-tune on user data
- Personalized categorization
- Custom suggestions

**Model Sharing:**
- Share custom models
- Community models
- Model marketplace

## AI API

### Rust API

```rust
use clipboard_ai::{AIProvider, LocalAI, CloudAI};

// Local AI
let ai = LocalAI::new();
let categories = ai.categorize("Hello, World!")?;

// Cloud AI
let ai = CloudAI::new("openai", "api_key");
let summary = ai.summarize("Long text...").await?;
```

### Plugin API

**WASM Plugin Access:**
```rust
#[no_mangle]
pub extern "C" fn openpaste_ai_categorize(
    content_ptr: *const u8,
    content_len: usize,
    out_ptr: *mut u8,
    out_len: *mut usize
) -> i32 {
    // Call AI categorization
    // Return categories
}
```

## Troubleshooting

### Common Issues

**Cloud AI Not Working:**
- Check API key
- Check internet connection
- Check API quota
- Check provider status

**Local AI Slow:**
- Check system resources
- Use smaller model
- Enable GPU acceleration

**Poor Results:**
- Try different provider
- Adjust prompts
- Use hybrid mode

### Debugging

**Enable Logging:**
```json
{
  "ai": {
    "debug": true,
    "log_requests": true,
    "log_responses": true
  }
}
```

## References

### AI Libraries

**Rust:**
- [candle](https://github.com/huggingface/candle)
- [llm](https://github.com/rustformers/llm)
- [burn](https://github.com/tracel-ai/burn)

**Python:**
- [OpenAI](https://github.com/openai/openai-python)
- [LangChain](https://github.com/langchain-ai/langchain)

### Models

**Open Source:**
- Llama 2
- Mistral
- Gemma
- Phi

### Documentation

**OpenAI:**
- [API Documentation](https://platform.openai.com/docs)

**Anthropic:**
- [API Documentation](https://docs.anthropic.com)

**LocalAI:**
- [Ollama](https://ollama.ai)
