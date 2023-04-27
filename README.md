# interactive-parse

**An interactive parser for JsonSchema types.**

---

## Demo

---

![](https://i.imgur.com/AVneDxD.gif)

---

## Usage

---

```rust
    // Make sure you add these derives to your type
    #[derive(JsonSchema, Deserialize, Debug)]
    struct Git {
        subcommand: SubCommand,
        /// Using doc comments like these will add hints to the prompt
        arg: String
    }

    #[derive(JsonSchema, Deserialize, Debug)]
    enum SubCommand {
        Commit {
            /// interactive-parse automatically handles
            /// any type that you can throw at it,
            /// like options
            message: Option<String>
        },
        Clone {
            /// vecs, and more!
            address: Vec<String>
        }
    }

    // Bring the relevant traits into scope
    use interactive_parse::{InteractiveParseObj, InteractiveParseVal};

    fn main() {
        // Parse the type to an object
        let git = Git::parse_to_obj().unwrap();

        // Or to a json value
        let value = Git::parse_to_val().unwrap();
        
        println!("{:?}", git);   
    }
```
---

## Cool features

---

hitting `Esc` during a multi-stage prompt will undo the last input and revert to the previous sub prompt. This feature is new and may skip back multiple prompts in some cases, but still very useful. Do to the recursive nature of this crate, undoing is very non-trivial.

---
## Looking for others to contribute

---

This is a simple approach at getting JsonSchema types to parse interactively using inquire. If you make improvements to this please submit a PR, and if you have any issues or bugs please submit an issue. I'm currently actively maintaining this project as a personal development tool.

In particular, this crate needs proper error handling. In most cases the crate will panic if it encounters an issue parsing. This is not ideal and could definitely be improved.

---
