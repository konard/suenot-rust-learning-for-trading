# Experiments Folder - DEPRECATED

⚠️ **This folder has been reorganized!**

All chapter-specific experiments have been moved to their respective chapter folders:

```
chapters/
├── 114-testing-errors-verifying-handling/experiments/
├── 117-errors-async-code/experiments/
├── 118-pattern-fail-fast/experiments/
├── 132-serde-rename/experiments/
├── 162-arc-mutex-shared-state/experiments/
├── 170-crossbeam-advanced-concurrency/experiments/
├── 175-thread-pool-limiting-parallelism/experiments/
├── 213-why-db-data-persistence/experiments/
├── 289-results-visualization/experiments/
├── 294-overfitting-strategy-optimization/experiments/
├── 295-cross-validation-strategies/experiments/
├── 299-multi-instrument-testing/experiments/
├── 303-documenting-results/experiments/
└── 309-string-vs-str-hot-paths/experiments/
```

## New Structure

Going forward, all experiments should be placed in:
```
chapters/<chapter-number-name>/experiments/
```

This prevents merge conflicts and keeps code organized with relevant chapters.

## Template Files

The remaining files in this folder serve as templates:
- `Cargo.toml` - Base configuration
- `.gitignore` - Ignore patterns
