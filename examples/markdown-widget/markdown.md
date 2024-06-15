# Introduction to Markdown

Markdown is a lightweight markup language with plain text formatting syntax. Its design allows it to be converted to many output formats, but the original tool by the same name only supports HTML. Markdown is often used to format readme files, for writing messages in online discussion forums, and to create rich text using a plain text editor.

## Benefits of Markdown

Markdown offers several advantages:

1. **Simplicity**: Markdown's syntax is simple and easy to learn.
2. **Readability**: Even without formatting, Markdown documents are readable in plain text.
3. **Portability**: Being plain text, Markdown files can be opened and edited in any text editor.
4. **Flexibility**: It can be converted to various formats including HTML, PDF, and more.

## Basic Syntax

Here are some basic Markdown syntax elements:

- **Headings**: Use `#` for headings. More `#` characters indicate smaller headings.
- **Bold**: Use `**` or `__` for bold text. Example: `**bold text**`
- **Italic**: Use `*` or `_` for italic text. Example: `*italic text*`
- **Lists**: Use `-` or `*` for unordered lists and numbers for ordered lists.

## Example

# Heading One
## Heading Two
### Heading Three
#### Heading Four
##### Heading Five
###### Heading Six

This is a paragraph with **bold text** and *italic text*.

> This is a block quote

- Item 1
- Item 2
- Item 3

1. First
2. Second
3. Third

# Example Markdown Document

This is a sample markdown document demonstrating how to include an embedded code snippet.


## Code Snippet

Below is an example of a simple Python function that adds two numbers:

```python
def add(a, b):
    """
    This function takes two numbers and returns their sum.
    
    Parameters:
    a (int or float): The first number.
    b (int or float): The second number.
    
    Returns:
    int or float: The sum of the two numbers.
    """
    return a + b

# Example usage:
result = add(3, 5)
print(f"The sum of 3 and 5 is {result}")
```

## and this is rust

```rust
fn parse_span(span: Span) -> String {
    let mut result = String::new();

    match span {
        markdown::Span::Break => result.push('\n'),
        markdown::Span::Text(text) => result.push_str(&text),
        markdown::Span::Code(code) => result.push_str(&code),
        markdown::Span::Link(_, _, _) => {}
        markdown::Span::Image(_, _, _) => {}
        markdown::Span::Emphasis(emphasis) => {
            for span in emphasis {
                result.push_str(&parse_span(span));
            }
        }
        markdown::Span::Strong(strong) => {
            for span in strong {
                result.push_str(&parse_span(span));
            }
        }
    }
    
}```

