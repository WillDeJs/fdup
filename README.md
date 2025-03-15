# Description
Personal and simple project for finding duplicate files in a directory. File comparison only happens via hashing. No sophisticated file comparison algorithm is implemented.

> [!NOTE] 
> This is just a personal project.
> Not intended for any production usage.

# Compile
Requires rust to be intalled and setup.
```
cargo.exe build --release
```

Executable located under:
```
target\release\fdbup.exe
```

# Usage
```
fdup.exe --path .
```

Alternatively run from cargo:
```
cargo run --release -- --path .
```