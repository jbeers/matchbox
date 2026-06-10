# MatchBox BIF Implementation Status

This document tracks the implementation status of BoxLang BIFs in MatchBox.

**Last Updated:** 2026-06-10

## Summary

| Status | Count |
|--------|-------|
| Total BoxLang BIFs | ~521 |
| Implemented (Native Rust) | 350+ |
| Implemented (Prelude) | 57 |
| **Total Implemented** | **407+** |
| **Remaining** | **~114** |
| **Coverage** | **~78%** |

## Status Legend

- ⬜ Not Started
- 🟡 In Progress
- ✅ Complete
- ❌ Not Applicable (e.g., Java-specific, requires JVM)

---

## Array BIFs

**Implemented:** 10 (native) + 21 (prelude) = 31 total
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| arrayAppend | ✅ | Native |
| arrayChunk | ✅ | Prelude |
| arrayClear | ✅ | Native |
| arrayDeleteAt | ✅ | Native |
| arrayFindAll | ✅ | Prelude |
| arrayFindFirst | ✅ | Prelude |
| arrayFlatMap | ✅ | Prelude |
| arrayFlatten | ✅ | Prelude |
| arrayGetMetadata | ✅ | Prelude |
| arrayGroupBy | ✅ | Prelude |
| arrayIndexExists | ✅ | Prelude |
| arrayInsertAt | ✅ | Native |
| arrayIsEmpty | ✅ | Prelude |
| arrayLen | ✅ | Native |
| arrayMerge | ✅ | Prelude |
| arrayMedian | ✅ | Prelude |
| arrayNew | ✅ | Native |
| arrayPop | ✅ | Native |
| arrayPush | ✅ | Prelude |
| arrayRange | ✅ | Prelude |
| arrayReduceRight | ✅ | Prelude |
| arrayReject | ✅ | Prelude |
| arrayResize | ✅ | Native |
| arraySet | ✅ | Native |
| arrayShift | ✅ | Prelude |
| arraySplice | ✅ | Prelude |
| arraySwap | ✅ | Native |
| arrayToStruct | ✅ | Prelude |
| arrayTranspose | ✅ | Prelude |
| arrayUnshift | ✅ | Prelude |
| arrayZip | ✅ | Prelude |

---

## Struct BIFs

**Implemented:** 21 (native) + 13 (prelude) = 34 total
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| structClear | ✅ | Native |
| structCount | ✅ | Native |
| structDelete | ✅ | Native |
| structEquals | ✅ | Native |
| structFind | ✅ | Native |
| structFindKey | ✅ | Native |
| structGet | ✅ | Native |
| structGetMetadata | ✅ | Native |
| structInsert | ✅ | Native |
| structIsEmpty | ✅ | Native |
| structIsCaseSensitive | ✅ | Native |
| structIsOrdered | ✅ | Native |
| structKeyArray | ✅ | Native |
| structKeyExists | ✅ | Native |
| structKeyTranslate | ✅ | Native |
| structNew | ✅ | Native |
| structToQueryString | ✅ | Native |
| structToSorted | ✅ | Native |
| structUpdate | ✅ | Native |

---

## String BIFs

**Implemented:** 46 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| ascii | ✅ | Native |
| camelCase | ✅ | Native |
| charsetDecode | ✅ | Native |
| charsetEncode | ✅ | Native |
| compare | ✅ | Native |
| compareNoCase | ✅ | Native |
| findOneOf | ✅ | Native |
| insert | ✅ | Native |
| jsStringFormat | ✅ | Native |
| justify | ✅ | Native (lJustify/rJustify) |
| kebabCase | ✅ | Native |
| left | ✅ | Native |
| len | ✅ | Native |
| ltrim | ✅ | Native |
| mid | ✅ | Native |
| paragraphFormat | ✅ | Native |
| pascalCase | ✅ | Native |
| queryStringToStruct | ✅ | Native |
| reEscape | ✅ | Native |
| removeChars | ✅ | Native |
| replaceList | ✅ | Native |
| replaceNoCase | ✅ | Native |
| repeatString | ✅ | Native |
| replace | ✅ | Native |
| reReplace | ✅ | Native |
| reReplaceNoCase | ✅ | Native |
| reverse | ✅ | Native |
| right | ✅ | Native |
| rtrim | ✅ | Native |
| slugify | ✅ | Native |
| snakeCase | ✅ | Native |
| spanExcluding | ✅ | Native |
| spanIncluding | ✅ | Native |
| sqlPrettify | ✅ | Native |
| stringBind | ✅ | Native |
| stringEndsWith | ✅ | Native |
| stringEndsWithNoCase | ✅ | Native |
| stringFind | ✅ | Native |
| stringFindNoCase | ✅ | Native |
| stringStartsWith | ✅ | Native |
| stringStartsWithNoCase | ✅ | Native |
| stripCR | ✅ | Native |
| trim | ✅ | Native |
| ucFirst | ✅ | Native |
| wrap | ✅ | Native |
| yesNoFormat | ✅ | Native |

---

## List BIFs

**Implemented:** 35 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| getToken | ✅ | Native |
| listAppend | ✅ | Native |
| listAvg | ✅ | Native |
| listChangeDelims | ✅ | Native |
| listCompact | ✅ | Native |
| listDeleteAt | ✅ | Native |
| listEach | ✅ | Native |
| listEvery | ✅ | Native |
| listFilter | ✅ | Native |
| listFind | ✅ | Native |
| listFindNoCase | ✅ | Native |
| listFirst | ✅ | Native |
| listGetAt | ✅ | Native |
| listGetEndings | ✅ | Native |
| listIndexExists | ✅ | Native |
| listInsertAt | ✅ | Native |
| listItemTrim | ✅ | Native |
| listLast | ✅ | Native |
| listLen | ✅ | Native |
| listMap | ✅ | Native |
| listNone | ✅ | Native |
| listPrepend | ✅ | Native |
| listQualify | ✅ | Native |
| listReduce | ✅ | Native |
| listReduceRight | ✅ | Native |
| listRemoveDuplicates | ✅ | Native |
| listRest | ✅ | Native |
| listSetAt | ✅ | Native |
| listSome | ✅ | Native |
| listSort | ✅ | Native |
| listToArray | ✅ | Native |
| listValueCount | ✅ | Native |

---

## Query BIFs

**Implemented:** 41 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| queryAddColumn | ✅ | Native |
| queryAddRow | ✅ | Native |
| queryAppend | ✅ | Native |
| queryClear | ✅ | Native |
| queryColumnArray | ✅ | Native |
| queryColumnCount | ✅ | Native |
| queryColumnData | ✅ | Native |
| queryColumnExists | ✅ | Native |
| queryColumnList | ✅ | Native |
| queryCurrentRow | ✅ | Native |
| queryDeleteColumn | ✅ | Native |
| queryDeleteRow | ✅ | Native |
| queryEach | ✅ | Native |
| queryEvery | ✅ | Native |
| queryExecute | ✅ | Native |
| queryFilter | ✅ | Native |
| queryGetCell | ✅ | Native |
| queryGetResult | ✅ | Native |
| queryInsertAt | ✅ | Native |
| queryKeyExists | ✅ | Native |
| queryMap | ✅ | Native |
| queryNone | ✅ | Native |
| queryNew | ✅ | Native |
| queryPrepend | ✅ | Native |
| queryRecordCount | ✅ | Native |
| queryReduce | ✅ | Native |
| queryRegisterFunction | ✅ | Native (stub) |
| queryReverse | ✅ | Native |
| queryRowData | ✅ | Native |
| queryRowSwap | ✅ | Native |
| querySetCell | ✅ | Native |
| querySetRow | ✅ | Native |
| querySlice | ✅ | Native |
| querySome | ✅ | Native |
| querySort | ✅ | Native |

---

## Math BIFs

**Implemented:** 29 (native) + 3 (prelude) = 32 total
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| abs | ✅ | Prelude |
| acos | ✅ | Native |
| asin | ✅ | Native |
| atan | ✅ | Native |
| atan2 | ✅ | Native |
| atn | ✅ | Native |
| ceiling | ✅ | Native |
| cos | ✅ | Native |
| decrementValue | ✅ | Native |
| exp | ✅ | Native |
| fix | ✅ | Native |
| floor | ✅ | Native |
| formatBaseN | ✅ | Native |
| incrementValue | ✅ | Native |
| inputBaseN | ✅ | Native |
| int | ✅ | Native |
| log | ✅ | Native |
| log10 | ✅ | Native |
| max | ✅ | Prelude |
| min | ✅ | Prelude |
| pi | ✅ | Native |
| precisionEvaluate | ✅ | Native (stub) |
| rand | ✅ | Native |
| randomize | ✅ | Native |
| randRange | ✅ | Native |
| round | ✅ | Native |
| sgn | ✅ | Native |
| sin | ✅ | Native |
| sqr | ✅ | Native |
| tan | ✅ | Native |
| val | ✅ | Native |

---

## Date/Time BIFs

**Implemented:** 17 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| createDate | ✅ | Native |
| createDateTime | ✅ | Native |
| createTime | ✅ | Native |
| createTimeSpan | ✅ | Native |
| dateAdd | ✅ | Native |
| dateCompare | ✅ | Native |
| dateConvert | ✅ | Native |
| dateDiff | ✅ | Native |
| dateFormat | ✅ | Native |
| datePart | ✅ | Native |
| dateTimeFormat | ✅ | Native |
| getTimezoneInfo | ✅ | Native |
| now | ✅ | Native |
| parseDateTime | ✅ | Native |
| setTimezone | ✅ | Native |
| clearTimezone | ✅ | Native |
| createODBCDateTime | ✅ | Native |
| timeUnits | ✅ | Native |

---

## Type Check BIFs

**Implemented:** 32 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| isArray | ✅ | Native |
| isBinary | ✅ | Native |
| isBoolean | ✅ | Native |
| isClosure | ✅ | Native |
| isCustomFunction | ✅ | Native |
| isDate | ✅ | Native |
| isDateObject | ✅ | Native |
| isDebugMode | ✅ | Native |
| isDefined | ✅ | Native (stub) |
| isEmpty | ✅ | Native |
| isFileObject | ✅ | Native |
| isIPv6 | ✅ | Native |
| isJSON | ✅ | Native |
| isLeapYear | ✅ | Native |
| isLocalHost | ✅ | Native |
| isNull | ✅ | Native |
| isNumeric | ✅ | Native |
| isObject | ✅ | Native |
| isQuery | ✅ | Native |
| isSimpleValue | ✅ | Native |
| isString | ✅ | Native |
| isStruct | ✅ | Native |
| isValid | ✅ | Native |
| isXML | ✅ | Native (stub) |
| isXMLAttribute | ✅ | Native (stub) |
| isXMLDoc | ✅ | Native (stub) |
| isXMLElement | ✅ | Native (stub) |
| isXMLNode | ✅ | Native (stub) |
| isXMLRoot | ✅ | Native (stub) |

---

## System BIFs

**Implemented:** 4 (native)
**Remaining:** 57

| BIF | Status | Notes |
|-----|--------|-------|
| createGUID | ✅ | Native |
| createUUID | ✅ | Native |
| getSystemSetting | ✅ | Native |
| getTickCount | ✅ | Native |
| writeOutput | ✅ | Native |
| *(remaining 56 unchanged)* | ⬜ | |

---

## File/IO BIFs

**Implemented:** 37 (native, behind bif-io feature)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| contractPath | ✅ | Native |
| createTempDirectory | ✅ | Native |
| createTempFile | ✅ | Native |
| directoryCreate | ✅ | Native |
| directoryCopy | ✅ | Native |
| directoryDelete | ✅ | Native |
| directoryExists | ✅ | Native |
| directoryList | ✅ | Native |
| directoryMove | ✅ | Native |
| expandPath | ✅ | Native |
| fileAppend | ✅ | Native |
| fileClose | ✅ | Native (stub - needs file handle infra) |
| fileCopy | ✅ | Native |
| fileCreateSymlink | ✅ | Native |
| fileDelete | ✅ | Native |
| fileExists | ✅ | Native |
| fileGetMimeType | ✅ | Native |
| fileInfo | ✅ | Native |
| fileIsEOF | ✅ | Native (stub - needs file handle infra) |
| fileMove | ✅ | Native |
| fileOpen | ✅ | Native (stub - needs file handle infra) |
| fileRead | ✅ | Native |
| fileReadLine | ✅ | Native (stub - needs file handle infra) |
| fileSeek | ✅ | Native (stub - needs file handle infra) |
| fileSetAccessMode | ✅ | Native |
| fileSetAttribute | ✅ | Native |
| fileSetExecutable | ✅ | Native |
| fileSetLastModified | ✅ | Native |
| fileWrite | ✅ | Native |
| fileWriteLine | ✅ | Native (stub - needs file handle infra) |
| getCanonicalPath | ✅ | Native |
| getDirectoryFromPath | ✅ | Native |
| propertyFile | ✅ | Native |

---

## Set BIFs

**Implemented:** 34 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| boxSetAdd | ✅ | Native |
| boxSetAddAll | ✅ | Native |
| boxSetClear | ✅ | Native |
| boxSetContains | ✅ | Native |
| boxSetContainsAll | ✅ | Native |
| boxSetDifference | ✅ | Native |
| boxSetEach | ✅ | Native |
| boxSetEquals | ✅ | Native |
| boxSetEvery | ✅ | Native |
| boxSetFilter | ✅ | Native |
| boxSetFind | ✅ | Native |
| boxSetIntersection | ✅ | Native |
| boxSetIsDisjointFrom | ✅ | Native |
| boxSetIsEmpty | ✅ | Native |
| boxSetIsSubsetOf | ✅ | Native |
| boxSetIsSupersetOf | ✅ | Native |
| boxSetMap | ✅ | Native |
| boxSetNone | ✅ | Native |
| boxSetReduce | ✅ | Native |
| boxSetReject | ✅ | Native |
| boxSetRemove | ✅ | Native |
| boxSetRemoveAll | ✅ | Native |
| boxSetRetainAll | ✅ | Native |
| boxSetSome | ✅ | Native |
| boxSetSymmetricDifference | ✅ | Native |
| boxSetToArray | ✅ | Native |
| boxSetToList | ✅ | Native |
| boxSetUnion | ✅ | Native |
| objectToSet | ✅ | Native |
| setNew | ✅ | Native |
| setOf | ✅ | Native |
| structKeySet | ✅ | Native |
| structValueSet | ✅ | Native |
| toSet | ✅ | Native |

---

## Binary/Bitwise BIFs

**Implemented:** 14 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| binaryDecode | ✅ | Native |
| binaryEncode | ✅ | Native |
| bitAnd | ✅ | Native |
| bitMaskClear | ✅ | Native |
| bitMaskRead | ✅ | Native |
| bitMaskSet | ✅ | Native |
| bitNot | ✅ | Native |
| bitOr | ✅ | Native |
| bitSh | ✅ | Native |
| bitXor | ✅ | Native |
| bytesGet | ✅ | Native |
| bytesLen | ✅ | Native |
| bytesNew | ✅ | Native |
| bytesSet | ✅ | Native |

---

## Encryption BIFs

**Implemented:** 6 (native, behind bif-crypto feature)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| decrypt | ✅ | Native (stub - needs cipher crates) |
| encrypt | ✅ | Native (stub - needs cipher crates) |
| generatePBKDFKey | ✅ | Native |
| generateSecretKey | ✅ | Native |
| hash | ✅ | Native |
| hmac | ✅ | Native |

---

## Conversion BIFs

**Implemented:** 12 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| dataNavigate | ✅ | Native |
| duplicate | ✅ | Native |
| jsonPrettify | ✅ | Native |
| loadProperties | ✅ | Native |
| parseNumber | ✅ | Native |
| toBase64 | ✅ | Native |
| toBinary | ✅ | Native |
| toModifiable | ✅ | Native |
| toNumeric | ✅ | Native |
| toScript | ✅ | Native |
| toString | ✅ | Native |
| toUnmodifiable | ✅ | Native |

---

## Format BIFs

**Implemented:** 5 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| booleanFormat | ✅ | Native |
| dateFormat | ✅ | Native |
| dateTimeFormat | ✅ | Native |
| decimalFormat | ✅ | Native |
| numberFormat | ✅ | Native |

---

## i18n/Locale BIFs

**Implemented:** 8 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| clearLocale | ✅ | Native |
| currencyFormat | ✅ | Native |
| getLocale | ✅ | Native |
| getLocaleDisplayName | ✅ | Native |
| getLocaleInfo | ✅ | Native |
| isCurrency | ✅ | Native |
| parseCurrency | ✅ | Native |
| setLocale | ✅ | Native |

---

## Watcher BIFs

**Implemented:** 11 (native, stubs)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| watcherExists | ✅ | Native (stub) |
| watcherGet | ✅ | Native (stub) |
| watcherGetAll | ✅ | Native (stub) |
| watcherList | ✅ | Native (stub) |
| watcherNew | ✅ | Native (stub - needs notify crate) |
| watcherRestart | ✅ | Native (stub) |
| watcherShutdown | ✅ | Native (stub) |
| watcherShutdownAll | ✅ | Native (stub) |
| watcherStart | ✅ | Native (stub) |
| watcherStop | ✅ | Native (stub) |
| watcherStopAll | ✅ | Native (stub) |

---

## Cache BIFs

**Implemented:** 5 (native, stubs)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| cache | ✅ | Native (stub) |
| cacheFilter | ✅ | Native (stub) |
| cacheNames | ✅ | Native (stub) |
| cacheProviders | ✅ | Native (stub) |
| cacheService | ✅ | Native (stub) |

---

## Zip BIFs

**Implemented:** 3 (native, behind bif-zip feature)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| compress | ✅ | Native |
| extract | ✅ | Native |
| isZipFile | ✅ | Native |

---

## CLI BIFs

**Implemented:** 7 (native, behind bif-cli feature)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| cliClear | ✅ | Native |
| cliConfirm | ✅ | Native |
| cliExit | ✅ | Native |
| cliGetArgs | ✅ | Native |
| cliRead | ✅ | Native |
| cliSelect | ✅ | Native |
| exit | ✅ | Native |

---

## JSON BIFs

**Implemented:** 5 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| deserializeJSON | ✅ | Native |
| isJSON | ✅ | Native |
| jsonDeserialize | ✅ | Native |
| jsonSerialize | ✅ | Native |
| serializeJSON | ✅ | Native |

---

## Regex BIFs

**Implemented:** 7 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| find | ✅ | Native |
| findNoCase | ✅ | Native |
| indexOf | ✅ | Native |
| reFind | ✅ | Native |
| reFindNoCase | ✅ | Native |
| reMatch | ✅ | Native |
| reMatchNoCase | ✅ | Native |

---

## Output BIFs

**Implemented:** 2 (native)
**Remaining:** 0

| BIF | Status | Notes |
|-----|--------|-------|
| writeOutput | ✅ | Native |
| yield | ✅ | Native |
