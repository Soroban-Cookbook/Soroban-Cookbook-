# Interactive Documentation Features - Design

## Architecture Overview

The interactive documentation features will be implemented using mdBook's built-in capabilities combined with custom CSS and minimal JavaScript. The approach prioritizes:
- Minimal dependencies
- Progressive enhancement (works without JS)
- Performance
- Maintainability

```
mdBook Content (Markdown)
    ↓
mdBook Processors & Plugins
    ↓
Custom HTML/CSS/JS Layer
    ↓
Interactive Documentation
```

## Component Design

### 1. Code Syntax Highlighting

**Implementation Approach:**
- Use mdBook's built-in `highlight.js` (already configured)
- Ensure Rust language is properly supported
- Add theme files for light/dark modes
- Support line numbering with CSS

**Key Features:**
```markdown
```rust
fn main() {
    println!("Hello, world!");
}
```

```markdown  
```rust,linenos
fn main() {
    println!("Hello, world!");
}
```

**Technical Details:**
- mdBook uses `highlight.js` for client-side highlighting
- Configure in `book.toml`:
  ```toml
  [output.html]
  default-theme = "light"
  dark-theme = "dark"
  ```
- Custom CSS for Rust-specific highlighting
- Line numbers via CSS counters

### 2. Copy-to-Clipboard Button

**Implementation Approach:**
- Add button overlay to each code block
- Use native JavaScript Clipboard API
- Minimal DOM manipulation

**Markup Pattern:**
```html
<pre class="copy-button-container">
  <button class="copy-button" onclick="copyCode(this)">Copy</button>
  <code class="language-rust">...</code>
</pre>
```

**JavaScript Implementation:**
```javascript
function copyCode(button) {
  const code = button.nextElementSibling.innerText;
  navigator.clipboard.writeText(code).then(() => {
    button.textContent = 'Copied!';
    setTimeout(() => {
      button.textContent = 'Copy';
    }, 2000);
  });
}
```

**Features:**
- Single line button (not intrusive)
- Visual feedback (button text change)
- Works cross-browser
- Keyboard accessible

### 3. Expandable/Collapsible Sections

**Markdown Syntax:**
```markdown
> ### Details (click to expand)
> 
> This content is initially hidden and can be expanded
> by clicking the header.
```

**HTML/CSS Approach:**
```html
<details>
  <summary>Click to expand</summary>
  <p>Hidden content here</p>
</details>
```

**Advantages:**
- Native HTML5 `<details>` element
- Works without JavaScript
- Keyboard accessible by default
- Semantic HTML

**Styling:**
- Custom CSS for arrow indicators
- Animation for smooth expand/collapse
- Accessible focus states

### 4. Tabs for Alternatives

**Markdown Syntax (using HTML):**
```html
<div class="tabs">
  <div class="tab-buttons">
    <button class="tab-button active" onclick="switchTab(event, 'rust')">Rust</button>
    <button class="tab-button" onclick="switchTab(event, 'js')">JavaScript</button>
  </div>
  
  <div id="rust" class="tab-content active">
    ```rust
    // Rust code
    ```
  </div>
  
  <div id="js" class="tab-content">
    ```javascript
    // JavaScript code
    ```
  </div>
</div>
```

**JavaScript:**
```javascript
function switchTab(event, tabName) {
  // Hide all tab contents
  const contents = document.querySelectorAll('.tab-content');
  contents.forEach(c => c.classList.remove('active'));
  
  // Deactivate all buttons
  const buttons = document.querySelectorAll('.tab-button');
  buttons.forEach(b => b.classList.remove('active'));
  
  // Show selected content and button
  document.getElementById(tabName).classList.add('active');
  event.target.classList.add('active');
}
```

**Features:**
- Clear visual indicators
- Keyboard navigation
- Mobile responsive
- Deep linking support (optional)

### 5. Feedback Widget

**Implementation Approach:**
- Sidebar or floating widget
- Simple reaction system (👍 👎)
- Optional text input
- Send to GitHub issues or external service

**Design:**
```html
<div class="feedback-widget">
  <p>Was this helpful?</p>
  <button onclick="submitFeedback('yes')">👍</button>
  <button onclick="submitFeedback('no')">👎</button>
  <a href="https://github.com/.../issues/new">Report issue</a>
</div>
```

**Features:**
- Non-intrusive
- Quick feedback (single click)
- Deep linking to relevant page
- Optional form for details

### 6. Documentation Structure

**File Organization:**
```
book/
├── src/
│   ├── SUMMARY.md (table of contents)
│   ├── README.md
│   ├── getting-started.md
│   ├── basics/
│   │   ├── storage.md
│   │   ├── authentication.md
│   │   └── events.md
│   ├── advanced/
│   │   ├── patterns.md
│   │   └── optimization.md
│   └── resources/
│       ├── glossary.md
│       └── links.md
└── book.toml
```

**Navigation Pattern:**
```markdown
# Page Title

[← Previous Page](../path/to/previous.md) | [Next Page →](../path/to/next.md)

## Table of Contents
- [Section 1](#section-1)
- [Section 2](#section-2)

...content...
```

## Integration Points

### mdBook Configuration

**book.toml additions:**
```toml
[output.html]
default-theme = "light"
dark-theme = "dark"
mathjax-support = false
fold-enable = true
highlight-theme = "sunny"

[build]
create-missing = false
```

### Custom CSS File

**book/theme/custom.css:**
```css
/* Copy button styling */
.copy-button {
  /* styles */
}

/* Details/summary styling */
details > summary {
  cursor: pointer;
}

/* Tabs styling */
.tabs {
  display: flex;
  flex-direction: column;
}

/* Feedback widget */
.feedback-widget {
  position: fixed;
  bottom: 20px;
  right: 20px;
}
```

### Custom JavaScript

**book/theme/interactive-docs.js:**
- Copy button functionality
- Tab switching
- Feedback submission
- Minimal overhead

## Page Layout Design

### Typical Page Structure
```
┌─────────────────────────────────────────┐
│ Header / Navigation                     │
├──────────────────┬──────────────────────┤
│ Sidebar          │ Main Content         │
│ (navigation)     │                      │
│                  │ [Page Title]         │
│                  │                      │
│                  │ [Interactive        │
│                  │  Elements]          │
│                  │                      │
│                  │ [Feedback Widget]   │
├──────────────────┴──────────────────────┤
│ Footer / Links                          │
└─────────────────────────────────────────┘
```

## Progressive Enhancement

### Baseline (no JavaScript)
- Syntax highlighting via mdBook
- Static code blocks with copy button (CSS only)
- Native `<details>` elements for expandable sections
- Basic navigation

### Enhanced (with JavaScript)
- Copy button with feedback
- Tab switching
- Feedback submission
- Improved accessibility

## Accessibility Considerations

### Keyboard Navigation
- Tab through interactive elements
- Enter/Space to activate buttons
- Arrow keys for tab switching
- Details/summary keyboard support (native)

### Screen Reader Support
- Semantic HTML (`<details>`, `<button>`, `<nav>`)
- ARIA labels for dynamic content
- Feedback widget announced
- Tab navigation announced

### Visual Accessibility
- High contrast syntax highlighting
- Clear focus indicators
- Sufficient button size
- Color not sole indicator (icons + text)

## Performance Optimization

### Load Time
- Syntax highlighting: built-in (no extra JS)
- Copy button: minimal JS (~2KB)
- Tab switching: minimal JS (~1KB)
- Feedback: async submission (non-blocking)

### Runtime Performance
- Copy button: < 50ms
- Tab switching: instant (CSS based)
- Syntax highlighting: pre-rendered
- No layout shifts

## Browser Compatibility

| Feature | Chrome | Firefox | Safari | Edge |
|---------|--------|---------|--------|------|
| Syntax Highlighting | ✓ | ✓ | ✓ | ✓ |
| Copy Button | ✓ | ✓ | ✓ | ✓ |
| Details/Summary | ✓ | ✓ | ✓ | ✓ |
| Tabs | ✓ | ✓ | ✓ | ✓ |
| Clipboard API | ✓ | ✓ | ✓ | ✓ |

## Maintenance & Updates

### For Content Creators
- Simple markdown syntax
- Clear documentation on patterns
- Copy-paste templates available
- Examples in existing docs

### For Developers
- Minimal custom code to maintain
- Leverage mdBook defaults
- Easy to update or extend
- Version controlled in repository

## Rollout Strategy

### Phase 1: Setup & Testing
- Configure mdBook with new themes
- Add custom CSS/JS files
- Test locally
- Verify mdBook build succeeds

### Phase 2: Implementation
- Add copy buttons to code blocks
- Add expandable sections
- Add tabs to key pages
- Add feedback widget

### Phase 3: Content Update
- Update 10-15 key pages
- Add interactive elements
- Test across browsers
- Collect user feedback

### Phase 4: Refinement
- Iterate based on feedback
- Optimize performance
- Expand to more pages
- Document patterns for contributors
