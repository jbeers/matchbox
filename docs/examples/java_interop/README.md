# Java Interop Example

This example demonstrates calling Java classes and methods from BoxLang using
MatchBox's experimental **JNI bridge**.

> **Important limitations:**
>
> - Java interop is available in **native builds only** (`--target native`).  
>   It is **not available** in WASM builds.
> - A compatible **JDK must be installed** on the host machine at runtime.
> - This feature is **highly experimental**. APIs may change between MatchBox releases.

## Running

Build a native binary and run it:

```bash
cd docs/examples/java_interop
matchbox --target native main.bxs
./main
```

You can also run in interpreter mode (still requires a JDK):

```bash
matchbox main.bxs
```

## What It Demonstrates

| Java class | Syntax |
|---|---|
| `java.lang.StringBuilder` | `import java:java.lang.StringBuilder as SB` |
| `java.util.ArrayList` | `import java:java.util.ArrayList as List` |
| `java.util.HashMap` | `import java:java.util.HashMap` |
| `java.lang.Math` | `java.new("java.lang.Math")` |

## Import Syntax

```boxlang
// Import with an alias (recommended — avoids the java: prefix on usage)
import java:java.util.ArrayList as List;
myList = new List()
myList.add("item")

// Import without an alias (use the bare class name)
import java:java.util.HashMap;
myMap = new HashMap()
myMap.put("key", "value")

// Direct instantiation without an import
sb = java.new("java.lang.StringBuilder", "Hello")
sb.append(", World!")
println(sb.toString())
```

## Limitations

| Feature | Status |
|---|---|
| Instantiate Java classes | ✅ |
| Call instance methods | ✅ |
| Static methods / static fields | ⚠️ Experimental |
| Java generics (type parameters) | ❌ |
| Java lambdas / streams | ❌ |
| WASM builds | ❌ Not supported |
| Running without a JDK installed | ❌ Not supported |

See [differences-from-boxlang.md](../../differences-from-boxlang.md) for the full
list of unsupported features across all build targets.
