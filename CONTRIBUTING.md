# Contributing to Markdown to docx Converter

Thank you for your interest in contributing to the Markdown to docx Converter! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Guidelines](#contributing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Community](#community)

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow. Please be respectful, inclusive, and constructive in all interactions.

### Our Standards

- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Ways to Contribute

- **Bug Reports**: Report issues you encounter
- **Feature Requests**: Suggest new features or improvements
- **Code Contributions**: Fix bugs or implement features
- **Documentation**: Improve or add documentation
- **Testing**: Add test cases or improve test coverage
- **Performance**: Optimize performance or memory usage
- **Examples**: Add usage examples or tutorials

### Before You Start

1. **Check existing issues**: Look for existing bug reports or feature requests
2. **Discuss major changes**: Open an issue to discuss significant changes before implementing
3. **Read the documentation**: Familiarize yourself with the project structure and goals

## Development Setup

### Prerequisites

- **Rust**: Version 1.70 or later
- **Node.js**: Version 18+ (for frontend development)
- **Git**: For version control
- **Docker**: Optional, for containerized development

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/yourusername/md2docx-converter
cd md2docx-converter

# Build the project
cargo build

# Run tests
cargo test

# Build frontend (optional)
cd frontend
npm install
npm run build
```

### Development Environment

```bash
# Enable debug logging
export RUST_LOG=debug

# Set up API key for testing natural language features
export OPENAI_API_KEY=your_test_api_key

# Run in development mode
cargo run --bin md2docx-server

# Run CLI tool
cargo run --bin md2docx-cli -- convert -i example.md -o output.docx
```

### Project Structure

```
├── src/                    # Rust source code
│   ├── bin/               # Binary executables
│   ├── config/            # Configuration management
│   ├── conversion/        # Core conversion engine
│   ├── docx/              # docx generation
│   ├── llm/               # LLM integration
│   ├── markdown/          # Markdown parsing
│   └── web/               # Web server and API
├── frontend/              # React frontend
├── tests/                 # Integration tests
├── examples/              # Example configurations
├── docs/                  # Documentation
└── Cargo.toml            # Rust dependencies
```

## Contributing Guidelines

### Issue Guidelines

#### Bug Reports

When reporting bugs, include:

- **Clear title**: Descriptive summary of the issue
- **Environment**: OS, Rust version, tool version
- **Steps to reproduce**: Minimal steps to reproduce the issue
- **Expected behavior**: What you expected to happen
- **Actual behavior**: What actually happened
- **Error messages**: Full error output with stack traces
- **Sample files**: Minimal markdown file that reproduces the issue

**Template:**
```markdown
## Bug Description
Brief description of the bug.

## Environment
- OS: macOS 13.0
- Rust version: 1.75.0
- Tool version: 0.1.0

## Steps to Reproduce
1. Create file with content: `# Test`
2. Run: `md2docx-cli convert -i test.md -o test.docx`
3. Error occurs

## Expected Behavior
Should create docx file successfully.

## Actual Behavior
Error: "Invalid markdown syntax"

## Error Output
```
[error output here]
```

## Sample Files
[Attach minimal test files]
```

#### Feature Requests

When requesting features, include:

- **Clear description**: What feature you want
- **Use case**: Why this feature is needed
- **Proposed solution**: How you think it should work
- **Alternatives**: Other solutions you've considered
- **Examples**: Mock-ups or examples of the desired behavior

### Code Contributions

#### Before Writing Code

1. **Open an issue**: Discuss the change before implementing
2. **Check existing work**: Look for related pull requests
3. **Understand the architecture**: Read the design document
4. **Start small**: Begin with small, focused changes

#### Development Workflow

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/your-feature-name`
3. **Make your changes**: Follow coding standards
4. **Add tests**: Ensure your changes are tested
5. **Update documentation**: Update relevant documentation
6. **Test thoroughly**: Run all tests and manual testing
7. **Commit your changes**: Use clear commit messages
8. **Push to your fork**: `git push origin feature/your-feature-name`
9. **Create pull request**: Submit PR with clear description

## Pull Request Process

### PR Requirements

- [ ] **Tests pass**: All existing tests continue to pass
- [ ] **New tests added**: New functionality is tested
- [ ] **Documentation updated**: Relevant docs are updated
- [ ] **Code formatted**: Code follows project style
- [ ] **No breaking changes**: Or clearly documented
- [ ] **Clear description**: PR describes what and why

### PR Template

```markdown
## Description
Brief description of changes made.

## Type of Change
- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed
- [ ] All tests pass

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

### Review Process

1. **Automated checks**: CI/CD pipeline runs tests
2. **Code review**: Maintainers review the code
3. **Feedback**: Address any requested changes
4. **Approval**: PR is approved by maintainers
5. **Merge**: PR is merged into main branch

## Coding Standards

### Rust Code Style

Follow standard Rust conventions:

```rust
// Use rustfmt for formatting
cargo fmt

// Use clippy for linting
cargo clippy -- -D warnings

// Follow naming conventions
struct MyStruct {
    field_name: String,
}

fn my_function() -> Result<(), Error> {
    // Implementation
}

// Use proper error handling
match result {
    Ok(value) => process_value(value),
    Err(error) => return Err(error.into()),
}
```

### Documentation Standards

```rust
/// Brief description of the function.
/// 
/// More detailed description if needed.
/// 
/// # Arguments
/// 
/// * `input` - Description of the input parameter
/// 
/// # Returns
/// 
/// Description of what is returned.
/// 
/// # Errors
/// 
/// Description of when errors occur.
/// 
/// # Examples
/// 
/// ```
/// let result = my_function("input");
/// assert!(result.is_ok());
/// ```
pub fn my_function(input: &str) -> Result<String, Error> {
    // Implementation
}
```

### Frontend Code Style

For React/TypeScript code:

```typescript
// Use Prettier for formatting
npm run format

// Follow TypeScript conventions
interface Props {
  markdown: string;
  onConvert: (result: ConversionResult) => void;
}

const MyComponent: React.FC<Props> = ({ markdown, onConvert }) => {
  // Implementation
};

export default MyComponent;
```

### Commit Message Format

Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(api): add natural language configuration endpoint

fix(parser): handle empty markdown files correctly

docs(readme): update installation instructions

test(conversion): add tests for table conversion
```

## Testing

### Test Categories

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test complete workflows
4. **Performance Tests**: Test performance characteristics

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_markdown_parsing

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test integration_tests

# With coverage
cargo tarpaulin --out html
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let input = "# Test";
        let result = parse_markdown(input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_functionality() {
        let result = async_function().await;
        assert_eq!(result, expected_value);
    }
}
```

### Test Guidelines

- **Test behavior, not implementation**: Focus on what the code does
- **Use descriptive names**: Test names should explain what is being tested
- **Keep tests simple**: One assertion per test when possible
- **Use test data**: Create reusable test fixtures
- **Mock external dependencies**: Don't rely on external services in tests

## Documentation

### Types of Documentation

1. **Code Documentation**: Inline comments and doc comments
2. **API Documentation**: REST API documentation
3. **User Documentation**: Usage guides and tutorials
4. **Developer Documentation**: Architecture and design docs

### Documentation Standards

- **Clear and concise**: Use simple, direct language
- **Examples included**: Provide code examples
- **Up to date**: Keep documentation current with code changes
- **Accessible**: Consider different skill levels

### Building Documentation

```bash
# Generate Rust documentation
cargo doc --open

# Build API documentation
# (API docs are in docs/API.md)

# Serve documentation locally
# (if using mdbook or similar)
```

## Performance Considerations

### Guidelines

- **Measure first**: Profile before optimizing
- **Memory efficiency**: Be mindful of memory usage
- **Async where appropriate**: Use async for I/O operations
- **Avoid premature optimization**: Focus on correctness first
- **Test performance**: Include performance tests for critical paths

### Profiling

```bash
# CPU profiling
cargo build --release
perf record target/release/md2docx-cli convert -i large_file.md -o output.docx
perf report

# Memory profiling
valgrind --tool=massif target/release/md2docx-cli convert -i file.md -o output.docx
```

## Security Considerations

### Guidelines

- **Input validation**: Validate all user input
- **Sanitization**: Sanitize data before processing
- **Error handling**: Don't leak sensitive information in errors
- **Dependencies**: Keep dependencies updated
- **Secrets**: Never commit API keys or secrets

### Security Review

- **Check for vulnerabilities**: Use `cargo audit`
- **Review dependencies**: Understand what dependencies do
- **Test edge cases**: Test with malicious input
- **Follow best practices**: Use established security patterns

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

- [ ] All tests pass
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version numbers updated
- [ ] Release notes prepared
- [ ] Security review completed

## Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Pull Requests**: Code contributions and reviews

### Getting Help

- **Documentation**: Check the docs/ directory
- **Examples**: Look at examples/ directory
- **Issues**: Search existing issues
- **Discussions**: Ask in GitHub Discussions

### Helping Others

- **Answer questions**: Help other users in discussions
- **Review PRs**: Provide constructive feedback on pull requests
- **Improve documentation**: Fix typos and add clarifications
- **Share examples**: Contribute usage examples

## Recognition

Contributors are recognized in:

- **CONTRIBUTORS.md**: List of all contributors
- **Release notes**: Major contributions mentioned
- **GitHub**: Contributor statistics and graphs

## Questions?

If you have questions about contributing:

1. **Check this document**: Most questions are answered here
2. **Search issues**: Someone might have asked already
3. **Open a discussion**: Ask in GitHub Discussions
4. **Contact maintainers**: Reach out to project maintainers

Thank you for contributing to the Markdown to docx Converter! Your contributions help make this tool better for everyone.