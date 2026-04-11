# Writing a lowfat plugin

Most commands can be filtered with a one-liner in `.lowfat` (see [README](../README.md#filtering-any-command)). Write a plugin only when pipeline config can't express what you need:

- **Per-subcommand logic** — different subcommands produce completely different output
- **Conditional output** — show "ok" when clean, show errors when not
- **Context-aware filtering** — different behavior based on flags or arguments

## Quick start

```sh
lowfat plugin new kubectl    # scaffold at ~/.lowfat/plugins/kubectl/kubectl-compact/
```

This creates:

```
~/.lowfat/plugins/kubectl/kubectl-compact/
  lowfat.toml     # manifest
  filter.sh       # your filter (edit this)
  samples/        # paste real output here for benchmarking
```

## Step 1: Edit the manifest

```toml
[plugin]
name = "kubectl-compact"
commands = ["kubectl"]
subcommands = ["get", "describe", "logs", "apply"]
```

- `commands` — the top-level command (e.g., `kubectl`)
- `subcommands` — which subcommands this plugin handles (omit to handle all)

## Step 2: Write the filter

Your script receives:

| Env var | Value | Example (`lowfat kubectl get pods -n kube-system -o wide`) |
|---------|-------|------|
| `$LOWFAT_COMMAND` | top-level command | `kubectl` |
| `$LOWFAT_SUBCOMMAND` | first argument | `get` |
| `$LOWFAT_ARGS` | all arguments joined by space | `get pods -n kube-system -o wide` |
| `$LOWFAT_LEVEL` | `lite`, `full`, or `ultra` | `full` |
| `$LOWFAT_EXIT_CODE` | command's exit code | `0` |
| stdin | raw command output | *(the full kubectl output)* |

`$LOWFAT_SUBCOMMAND` is just the first arg — good enough for most plugins. Use `$LOWFAT_ARGS` when you need to inspect deeper arguments (resource type, flags, output format, etc.).

Write filtered output to stdout. If you output nothing, lowfat passes through the original.

### Example: kubectl-compact/filter.sh

This shows how to use `$LOWFAT_ARGS` to handle resource-type-specific filtering:

```sh
#!/bin/sh
# kubectl-compact — compact kubectl output for LLM contexts

RAW=$(cat)
LEVEL="${LOWFAT_LEVEL:-full}"
SUB="${LOWFAT_SUBCOMMAND}"
ARGS="${LOWFAT_ARGS}"

# Extract resource type from args: "get pods -n foo" → "pods"
# Skip flags (starting with -) to find the resource type
RESOURCE=""
for arg in $ARGS; do
  case "$arg" in
    "$SUB") continue ;;           # skip subcommand itself
    -*) continue ;;               # skip flags
    *) RESOURCE="$arg"; break ;;  # first non-flag = resource type
  esac
done

# Detect -o json/yaml (structured output should pass through)
case "$ARGS" in
  *"-o json"*|*"-o yaml"*|*"--output json"*|*"--output yaml"*)
    echo "$RAW"
    exit 0
    ;;
esac

case "$SUB" in
  get)
    case "$RESOURCE" in
      pods|po)
        if [ "$LEVEL" = "ultra" ]; then
          # Header + non-Running pods (problems only)
          echo "$RAW" | awk 'NR==1 || !/Running/' | head -n 15
        else
          LIMIT=$( [ "$LEVEL" = "lite" ] && echo 60 || echo 30 )
          echo "$RAW" | cut -c1-100 | head -n "$LIMIT"
        fi
        ;;
      events|ev)
        if [ "$LEVEL" = "ultra" ]; then
          # Warning events only
          echo "$RAW" | awk 'NR==1 || /Warning/' | head -n 15
        else
          echo "$RAW" | tail -n 30
        fi
        ;;
      *)
        # Generic table: trim width, limit rows
        LIMIT=$( [ "$LEVEL" = "lite" ] && echo 60 || echo 30 )
        echo "$RAW" | cut -c1-120 | head -n "$LIMIT"
        ;;
    esac
    ;;
  describe)
    if [ "$LEVEL" = "ultra" ]; then
      echo "$RAW" | grep -E '^(Name:|Namespace:|Status:|Type:|Reason:|Message:|  [A-Z])' | head -n 20
    else
      LIMIT=$( [ "$LEVEL" = "lite" ] && echo 80 || echo 40 )
      echo "$RAW" | head -n "$LIMIT"
    fi
    ;;
  logs)
    case "$LEVEL" in
      ultra) echo "$RAW" | grep -iE 'error|warn|fatal|panic' | head -n 15 ;;
      lite)  echo "$RAW" | tail -n 60 ;;
      *)     echo "$RAW" | tail -n 30 ;;
    esac
    ;;
  apply)
    if [ "$LEVEL" = "ultra" ]; then
      echo "$RAW" | grep -E '(created|configured|unchanged|deleted)$' | head -n 15
    else
      echo "$RAW" | head -n 30
    fi
    ;;
  *)
    echo "$RAW" | head -n 30
    ;;
esac
```

### Example: terraform-compact/filter.sh

```sh
#!/bin/sh
# terraform-compact — compact terraform output for LLM contexts

RAW=$(cat)
LEVEL="${LOWFAT_LEVEL:-full}"
SUB="${LOWFAT_SUBCOMMAND}"

case "$SUB" in
  plan)
    if [ "$LEVEL" = "ultra" ]; then
      # Plan summary only
      SUMMARY=$(echo "$RAW" | grep -E '^Plan:|^No changes|^Error')
      echo "${SUMMARY:-terraform plan: ok}"
    else
      # Strip unchanged resources, keep creates/updates/destroys
      LIMIT=$( [ "$LEVEL" = "lite" ] && echo 80 || echo 40 )
      echo "$RAW" | grep -vE '^ +# .* will be read|no changes' | head -n "$LIMIT"
    fi
    ;;
  apply)
    if [ "$LEVEL" = "ultra" ]; then
      echo "$RAW" | grep -E '^(Apply complete!|Error:|module\.)' | head -n 10
    else
      LIMIT=$( [ "$LEVEL" = "lite" ] && echo 60 || echo 30 )
      echo "$RAW" | grep -vE '^\s*$|^  #' | head -n "$LIMIT"
    fi
    ;;
  init)
    if [ "$LEVEL" = "ultra" ]; then
      echo "$RAW" | grep -E '(successfully initialized|Upgrading|Error)' | head -n 5
    else
      echo "$RAW" | head -n 20
    fi
    ;;
  *)
    echo "$RAW" | head -n 30
    ;;
esac
```

### Example: pytest-compact/filter.sh

```sh
#!/bin/sh
# pytest-compact — compact pytest output for LLM contexts

RAW=$(cat)
LEVEL="${LOWFAT_LEVEL:-full}"

case "$LEVEL" in
  ultra)
    # Summary line + FAILED test names only
    echo "$RAW" | grep -E '^(FAILED|ERROR|=+ .+ =+$)' | head -n 15
    ;;
  lite)
    # Strip per-test progress dots, keep everything else
    echo "$RAW" | grep -vE '^\s*\.\.\.' | head -n 60
    ;;
  *)
    echo "$RAW" | grep -vE '^\s*\.\.\.' | head -n 30
    ;;
esac
```

## Step 3: Add sample data

Capture real output for benchmarking:

```sh
# Naming convention: <command>-<subcommand>-<level>.txt
kubectl get pods -A > ~/.lowfat/plugins/kubectl/kubectl-compact/samples/kubectl-get-full.txt
kubectl describe pod mypod > ~/.lowfat/plugins/kubectl/kubectl-compact/samples/kubectl-describe-full.txt
```

## Step 4: Benchmark

```sh
lowfat plugin bench kubectl-compact
```

Output:

```
Benchmark: kubectl-compact

  kubectl-get-full (full)         892 →     45 tokens  (-95%)
  kubectl-describe-full (full)    3401 →    122 tokens  (-96%)

  TOTAL                           4293 →    167 tokens  (-96%)
```

Aim for **80%+ savings** at `full` level while keeping all actionable information.

## Step 5: Test it

```sh
lowfat kubectl get pods
lowfat kubectl describe pod mypod
LOWFAT_LEVEL=ultra lowfat kubectl get pods -A
```

## Plugin design rules

1. **Errors are sacred.** Never filter out error messages. If exit code is non-zero, be conservative.
2. **ultra = what the AI needs to decide next.** Summary line, error count, pass/fail — nothing more.
3. **full = what a human would skim.** Strip noise (progress bars, download lines, repeated ok), keep structure.
4. **lite = gentle trim.** Just cut length, keep context.
5. **Default case matters.** Always handle unknown subcommands with a reasonable `head -n 30`.
6. **Empty output = passthrough.** If your filter outputs nothing, lowfat uses the original output.

## The pattern

Every plugin follows the same shape:

```sh
#!/bin/sh
RAW=$(cat)
LEVEL="${LOWFAT_LEVEL:-full}"
SUB="${LOWFAT_SUBCOMMAND}"

case "$SUB" in
  <subcommand>)
    if [ "$LEVEL" = "ultra" ]; then
      # Extract summary/errors only
    else
      LIMIT=$( [ "$LEVEL" = "lite" ] && echo 60 || echo 30 )
      # Strip noise, keep signal
      echo "$RAW" | grep -vE '<noise patterns>' | head -n "$LIMIT"
    fi
    ;;
  *)
    echo "$RAW" | head -n 30
    ;;
esac
```

## Advanced: Pipeline integration

Mix your plugin with built-in processors in `.lowfat`:

```
pipeline.kubectl = strip-ansi | kubectl-compact | truncate:100
pipeline.kubectl.error = strip-ansi | head
```

## Advanced: Manifest options

```toml
[plugin]
name = "kubectl-compact"
version = "0.1.0"
description = "Compact kubectl output"
author = "you"
commands = ["kubectl"]
subcommands = ["get", "describe", "logs", "apply"]

[runtime]
entry = "filter.sh"       # default entry point

[hooks]
on_install = "chmod +x filter.sh"

[pipeline]
pre = ["strip-ansi"]      # run before your filter
post = ["truncate"]        # run after your filter
```
