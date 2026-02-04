# Fortified LLM Client Documentation

This directory contains the Jekyll-based documentation site using the [just-the-docs](https://just-the-docs.com/) theme.

## Local Development

### Prerequisites

- Ruby 2.7 or later
- Bundler

### Setup

```bash
# Install dependencies
bundle install

# Serve locally
bundle exec jekyll serve

# View at http://localhost:4000/fortified-llm-client
```

### Making Changes

1. Edit `.md` files in subdirectories
2. Jekyll auto-rebuilds on file changes
3. Refresh browser to see updates

## Structure

```
docs/
├── _config.yml           # Jekyll configuration
├── Gemfile               # Ruby dependencies
├── index.md              # Landing page
├── getting-started/      # Installation and quick start
├── user-guide/           # CLI and library usage
├── architecture/         # Design and internals
├── guardrails/           # Security validation
├── examples/             # Code examples
├── advanced/             # Advanced topics
└── contributing/         # Development guidelines
```

## Deployment

GitHub Pages automatically builds and deploys Jekyll sites from the `/docs` directory on the `main` branch. No additional CI/CD configuration needed!

## Theme

This site uses [just-the-docs v0.8.2](https://github.com/just-the-docs/just-the-docs) for a clean, minimal aesthetic similar to mdBook.

## Navigation

Front matter controls navigation:

```yaml
---
layout: default
title: Page Title
parent: Parent Page  # Optional
nav_order: 1         # Determines sidebar order
has_children: true   # For parent pages
---
```

## Links

Use Liquid tags for internal links:

```markdown
[Getting Started]({{ site.baseurl }}{% link getting-started/index.md %})
```

This ensures links work in both local development and production.
