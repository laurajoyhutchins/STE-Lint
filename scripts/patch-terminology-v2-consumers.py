from pathlib import Path

path = Path("crates/ste-cli/src/main.rs")
text = path.read_text()
old = '''                    for term in profile.terms {
                        println!("- {} [{}]", term.term, serialized_label(&term.kind));
                    }
'''
new = '''                    for term in profile.terms {
                        let roles = term
                            .roles
                            .iter()
                            .map(serialized_label)
                            .collect::<Vec<_>>()
                            .join(", ");
                        println!("- {} [{}]", term.canonical, roles);
                    }
'''
if old not in text:
    raise SystemExit("expected profile output block not found")
path.write_text(text.replace(old, new))
