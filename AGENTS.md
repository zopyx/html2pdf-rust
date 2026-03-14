# HTML Form Element Support

## Overview

This document describes the HTML form element rendering support added to html2pdf-rs.

## Supported Form Elements

### Input Types

| Type | Support Status | Notes |
|------|---------------|-------|
| `text` | ✅ Full | Standard text input with border, background |
| `password` | ✅ Full | Displays as asterisks (•) for security |
| `email` | ✅ Full | Same styling as text input |
| `number` | ✅ Full | Narrower default width |
| `checkbox` | ✅ Full | Renders as ☑ or ☐ with checkmark |
| `radio` | ✅ Full | Renders as ◉ or ○ with filled circle |
| `date` | ✅ Full | Calendar-style input appearance |
| `time` | ✅ Full | Time picker appearance |
| `file` | ✅ Full | Shows "Choose file..." or filename with paperclip icon |
| `hidden` | ✅ Full | Skipped in rendering (as expected) |

### Other Elements

| Element | Support Status | Notes |
|---------|---------------|-------|
| `textarea` | ✅ Full | Multi-line text area with scroll appearance |
| `select` | ✅ Full | Dropdown with arrow indicator |
| `option` | ✅ Full | Part of select element |
| `button` | ✅ Full | Styled button with background |
| `label` | ✅ Full | Text with optional "for" association |

## Implementation Details

### Module Structure

```
src/layout/
├── form.rs          # New: Form element handling
├── box_model.rs     # Modified: Added FormBox to LayoutBox
├── flow.rs          # Modified: Layout for form controls
├── style.rs         # Modified: Default styles for form elements
├── mod.rs           # Modified: Export form module
└── ...

src/pdf/
├── mod.rs           # Modified: Added draw_form_control() methods
└── ...
```

### New Types

- `FormControlType`: Enum representing different form control types
- `FormBox`: Struct holding form control data (value, state, options, etc.)
- `SelectOption`: Struct for select dropdown options

### Box Model Changes

- Added `BoxType::Form` to represent form control boxes
- Added `form_data: Option<FormBox>` field to `LayoutBox`
- Form boxes are treated as inline-level elements

### Default Styling

Form elements receive sensible default styles:
- Border: 1px solid #999999
- Background: white (or #f0f0f0 for buttons)
- Padding: varies by element type
- Margins: 2px top/bottom for spacing

### Layout

Form controls are laid out using:
- Intrinsic dimensions based on control type
- CSS width/height if specified
- Proper inline/block context handling
- Margin/padding/border support

### PDF Rendering

Each form control type has specialized rendering:
- **Text inputs**: Rectangle with border, text content
- **Checkboxes**: Box with optional checkmark
- **Radio buttons**: Circle with optional filled center
- **Select**: Rectangle with dropdown arrow
- **Buttons**: Styled rectangle with centered text
- **Labels**: Plain text rendering

## Usage Example

```rust
use html2pdf::{html_to_pdf, Config};

let html = r#"
    <form>
        <label for="name">Name:</label>
        <input type="text" id="name" value="John Doe">
        
        <input type="checkbox" checked> Subscribe
        
        <button type="submit">Submit</button>
    </form>
"#;

let config = Config::default();
let pdf = html_to_pdf(html, &config)?;
```

## Testing

Run form element tests:
```bash
cargo test form_elements
```

Test file: `tests/form_elements.rs`

## Future Enhancements (Optional)

### Interactive PDF Forms

The current implementation renders form elements as static content. Future enhancements could include:

- **PDF AcroForm fields**: Add interactive text fields
- **PDF Checkboxes**: Interactive checkboxes that can be clicked
- **PDF Radio buttons**: Grouped radio button widgets
- **PDF Submit buttons**: Form submission actions
- **Field validation**: JavaScript validation in PDF

To implement interactive forms, the following would need to be added:
1. `pdf::FormField` types for different field types
2. Annotation dictionaries in PDF output
3. JavaScript actions for validation/submission
4. Form dictionary in PDF catalog

## CSS Support

Form elements support these CSS properties:
- `width`, `height`: Explicit dimensions
- `border`, `border-width`, `border-color`, `border-style`
- `background-color`
- `padding`, `margin`
- `color`: Text color
- `font-family`, `font-size`

Focus and hover states are not animated (PDF is static) but could be rendered as alternate appearances.
