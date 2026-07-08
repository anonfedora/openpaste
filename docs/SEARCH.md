# OpenPaste Search Design

## Overview

OpenPaste provides fast, intelligent search across clipboard history using SQLite FTS5 with custom ranking algorithms. Target latency is <20ms for searches across 10,000 items.

## Search Pipeline

```
User Query
    │
    ▼
Query Parsing (tokenize, normalize)
    │
    ▼
Query Transformation (expand, optimize)
    │
    ▼
FTS5 Search (execute MATCH query)
    │
    ▼
Result Ranking (BM25 + custom factors)
    │
    ▼
Result Filtering (type, date, tags, collection)
    │
    ▼
Result Highlighting (mark matches)
    │
    ▼
Return Results (sorted, paginated)
```

## Query Parsing

### Tokenization

**Input:** Raw user query string

**Process:**
1. Trim whitespace
2. Normalize Unicode (NFC form)
3. Remove diacritics (optional, based on settings)
4. Split into tokens
5. Identify special operators

**Example:**
```
Input: "  hello   world  "
Output: ["hello", "world"]
```

### Operators

**Phrase Search:**
```
"exact phrase"
```

**Exclusion:**
```
hello -world
```

**Field Search:**
```
app:chrome
window:github
tag:important
```

**Date Range:**
```
after:2024-01-01
before:2024-12-31
```

**Type Filter:**
```
type:text
type:image
```

**Boolean Operators:**
```
hello AND world
hello OR world
hello NOT world
```

### Query Normalization

**Lowercase:** All queries converted to lowercase for case-insensitive search

**Whitespace:** Multiple spaces collapsed to single space

**Special Characters:** Punctuation handled based on tokenizer settings

## Query Transformation

### Expansion

**Synonym Expansion:**
```
"clip" → "clipboard", "clip"
"paste" → "paste", "copy"
```

**Stemming:** (Optional, future enhancement)
```
"running" → "run"
"copied" → "copy"
```

**Fuzzy Matching:** (Optional, future enhancement)
```
"helol" → "hello" (within 1 edit distance)
```

### Optimization

**Stop Words:** Remove common words if they don't affect meaning
```
"the quick brown fox" → "quick brown fox"
```

**Query Simplification:** Remove redundant terms
```
"hello hello world" → "hello world"
```

**Index Hints:** Use appropriate indexes based on query type

## FTS5 Search

### MATCH Query

**Basic Search:**
```sql
SELECT ci.id, ci.content_preview, ci.created_at, fts.rank
FROM clipboard_items ci
JOIN clipboard_items_fts fts ON ci.id = fts.rowid
WHERE clipboard_items_fts MATCH ?
AND ci.deleted_at IS NULL
ORDER BY fts.rank
LIMIT 50;
```

**Phrase Search:**
```sql
WHERE clipboard_items_fts MATCH '"exact phrase"'
```

**Boolean Search:**
```sql
WHERE clipboard_items_fts MATCH 'hello AND world'
```

**Prefix Search:**
```sql
WHERE clipboard_items_fts MATCH 'hel*'
```

### FTS5 Configuration

**Tokenizer:**
```sql
tokenize="unicode61 remove_diacritics 1"
```

**Ranking:**
```sql
INSERT INTO clipboard_items_fts(clipboard_items_fts, rank) 
VALUES('rank', 'bm25(1.0, 0.75, 0.0, 0.0)');
```

**BM25 Parameters:**
- **k1**: 1.0 (term saturation parameter)
- **b**: 0.75 (length normalization)
- **0.0**: Not used for our schema

## Result Ranking

### BM25 Score

FTS5 provides BM25 score as base ranking.

### Custom Ranking Factors

**Recency Boost:**
```rust
fn recency_boost(created_at: i64) -> f64 {
    let age_hours = (now - created_at) / 3600000;
    1.0 / (1.0 + age_hours / 24.0) // Decay over days
}
```

**Frequency Boost:**
```rust
fn frequency_boost(access_count: i32) -> f64 {
    1.0 + (access_count as f64).log10()
}
```

**Pinned Boost:**
```rust
fn pinned_boost(pinned: bool) -> f64 {
    if pinned { 10.0 } else { 0.0 }
}
```

**Favorite Boost:**
```rust
fn favorite_boost(favorite: bool) -> f64 {
    if favorite { 5.0 } else { 0.0 }
}
```

**Source App Boost:** (Optional, configurable)
```rust
fn source_app_boost(source_app: &str, preferred_apps: &[&str]) -> f64 {
    if preferred_apps.contains(&source_app) { 2.0 } else { 0.0 }
}
```

### Combined Score

```rust
fn combined_score(
    bm25: f64,
    recency: f64,
    frequency: f64,
    pinned: f64,
    favorite: f64,
) -> f64 {
    bm25 * 1.0 + 
    recency * 0.5 + 
    frequency * 0.3 + 
    pinned * 10.0 + 
    favorite * 5.0
}
```

### Sorting

Results sorted by combined score descending.

## Result Filtering

### Type Filter

```sql
WHERE content_type IN ('text', 'html')
```

### Date Range Filter

```sql
WHERE created_at >= ? AND created_at <= ?
```

### Tag Filter

```sql
WHERE id IN (
    SELECT clipboard_item_id FROM clipboard_item_tags
    JOIN tags ON clipboard_item_tags.tag_id = tags.id
    WHERE tags.name IN (?)
)
```

### Collection Filter

```sql
WHERE collection_id = ?
```

### Source App Filter

```sql
WHERE source_app LIKE ?
```

### Pinned/Favorite Filter

```sql
WHERE pinned = 1
WHERE favorite = 1
```

## Result Highlighting

### Match Highlighting

**Algorithm:**
1. Identify matched terms in content
2. Wrap matches in highlight markers
3. Limit highlighted snippet length
4. Truncate with ellipsis if needed

**Example:**
```
Content: "The quick brown fox jumps over the lazy dog"
Query: "fox dog"
Result: "The quick brown <mark>fox</mark> jumps over the lazy <mark>dog</mark>"
```

**Implementation:**
```rust
fn highlight_matches(content: &str, query_terms: &[&str]) -> String {
    let mut result = content.to_string();
    for term in query_terms {
        let pattern = regex::Regex::new(&format!("(?i){}", regex::escape(term))).unwrap();
        result = pattern.replace_all(&result, "<mark>$0</mark>").to_string();
    }
    result
}
```

### Snippet Generation

**Context Window:** 50 characters before and after match

**Multiple Matches:** Show up to 3 matches per result

**Truncation:** Use "..." for truncated sections

## Search Performance

### Performance Targets

- **Search Latency:** <20ms for 10,000 items
- **Index Update:** <10ms per item
- **Query Parsing:** <1ms
- **Result Ranking:** <5ms for 50 results

### Optimization Strategies

**FTS Index:**
- Keep FTS index in memory (cache_size pragma)
- Use appropriate tokenizer
- Optimize BM25 parameters

**Query Optimization:**
- Use parameterized queries
- Limit result set (default 50)
- Use indexes for filters
- Avoid full table scans

**Caching:**
- Cache frequent searches
- Cache query parsing results
- Cache ranking calculations

**Connection Pooling:**
- Reuse database connections
- Set appropriate pool size

### Performance Monitoring

**Metrics:**
- Search latency (p50, p95, p99)
- Index update time
- Query complexity
- Result set size

**Alerting:**
- Alert if p95 latency > 50ms
- Alert if index update time > 100ms

## Advanced Search Features

### Regex Search

**Implementation:** (Future enhancement)
```sql
-- Use REGEXP operator (requires loadable extension)
WHERE content_preview REGEXP ?
```

**Fallback:** Use Rust regex library on retrieved results

### Fuzzy Search

**Implementation:** (Future enhancement)
- Levenshtein distance
- Phonetic matching (Soundex, Metaphone)
- Typo tolerance

**Library:** `strsim` or `fuzzy-matcher`

### Semantic Search

**Implementation:** (Future enhancement, see AI.md)
- Vector embeddings
- Cosine similarity
- Hybrid keyword + semantic search

### Natural Language Query

**Implementation:** (Future enhancement)
- Parse natural language queries
- Extract intent
- Transform to structured query

**Example:**
```
"images from yesterday" → type:image AND after:yesterday
"copied from chrome" → app:chrome
```

## Search API

### Search Request

```rust
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub content_types: Option<Vec<ContentType>>,
    pub after_date: Option<i64>,
    pub before_date: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub collection_id: Option<i64>,
    pub source_app: Option<String>,
    pub pinned_only: Option<bool>,
    pub favorite_only: Option<bool>,
}
```

### Search Response

```rust
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub query: String,
    pub latency_ms: u64,
}

pub struct SearchResult {
    pub id: i64,
    pub content_preview: String,
    pub highlighted_preview: String,
    pub content_type: ContentType,
    pub created_at: i64,
    pub source_app: Option<String>,
    pub score: f64,
    pub pinned: bool,
    pub favorite: bool,
    pub tags: Vec<String>,
}
```

## Search UI Integration

### Instant Search

- Trigger search on each keystroke
- Debounce by 150ms
- Show loading indicator
- Update results incrementally

### Keyboard Navigation

- **Arrow Up/Down:** Navigate results
- **Enter:** Select and copy
- **Escape:** Close search
- **Ctrl+K:** Focus search

### Search Suggestions

- Show recent searches
- Show popular searches
- Show query completions
- Show search history

### Search Filters UI

- Dropdown for content type
- Date picker for date range
- Tag selector
- Collection selector
- Toggle for pinned/favorite

## Search Analytics

### Tracked Metrics

- Search query frequency
- Search result click-through rate
- Zero-result searches
- Average search latency
- Most used filters

### Privacy

- No search query logging by default
- Opt-in analytics only
- No personal data in analytics
- Aggregate data only

## Testing

### Unit Tests

- Query parsing
- Query transformation
- Ranking algorithms
- Highlighting logic
- Filter application

### Integration Tests

- FTS search execution
- Database query performance
- End-to-end search pipeline
- Search API responses

### Performance Tests

- Search latency benchmarks
- Index update benchmarks
- Concurrent search load
- Large dataset performance

### Test Data

- Sample clipboard items (various types)
- Sample queries (simple, complex, edge cases)
- Expected results for each query

## Future Enhancements

### Tantivy Integration

**Why:** More advanced search features

**Migration Path:**
- Keep SQLite for storage
- Use Tantivy for search index
- Hybrid approach for transition

**Benefits:**
- Better fuzzy search
- More ranking options
- Faceted search
- Better performance at scale

### Machine Learning Ranking

**Why:** Improve relevance

**Approach:**
- Learn from user interactions
- Personalized ranking
- Context-aware ranking

### Search Plugins

**Why:** Extensible search

**API:**
- Custom ranking functions
- Custom filters
- Custom result transformers

## Search Configuration

### User Settings

```json
{
  "search": {
    "instant_search": true,
    "debounce_ms": 150,
    "highlight_matches": true,
    "fuzzy_search": false,
    "remove_diacritics": true,
    "max_results": 50,
    "recency_boost": 0.5,
    "frequency_boost": 0.3,
    "pinned_boost": 10.0,
    "favorite_boost": 5.0
  }
}
```

### Default Settings

- Instant search: enabled
- Debounce: 150ms
- Highlighting: enabled
- Fuzzy search: disabled
- Remove diacritics: enabled
- Max results: 50
- Recency boost: 0.5
- Frequency boost: 0.3
- Pinned boost: 10.0
- Favorite boost: 5.0

## Search Error Handling

### Error Types

**Invalid Query:**
- Return empty results
- Show error message to user
- Suggest query correction

**Database Error:**
- Log error
- Return cached results if available
- Show error message to user

**Timeout:**
- Return partial results
- Show timeout warning
- Retry in background

### Fallback Strategies

**FTS Unavailable:**
- Fall back to LIKE queries
- Slower but functional
- Warn user about degraded performance

**Index Corrupted:**
- Rebuild index
- Show progress to user
- Use cached results during rebuild

## Search Accessibility

### Keyboard Shortcuts

- **Ctrl/Cmd+K:** Focus search
- **Escape:** Clear search
- **Enter:** Submit search
- **Arrow Keys:** Navigate results

### Screen Reader Support

- Announce search results count
- Announce current selection
- Provide search status updates

### High Contrast

- Highlight matches with high contrast
- Ensure text readability
- Support system high contrast mode
