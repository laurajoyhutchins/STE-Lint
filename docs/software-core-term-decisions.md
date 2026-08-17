# software-core terminology decisions

Status: approved design basis for `software-core` profile version 3.

## Purpose

`software-core` is a shared software subject-field terminology base. It is not a supplemental general-English dictionary or a developer-jargon whitelist.

A term is admitted only when it names a specified software concept or irreducible software process that is stable across unrelated codebases and ordinary approved ASD-STE100 vocabulary would lose the required technical distinction. Corpus frequency can trigger review; it never establishes authority. Technical verbs receive the stricter test: prefer an approved general verb whenever it communicates the same operation accurately.

## Metalanguage boundary

This file is terminology **metalanguage**, not governed target technical prose. Definitions, admission rationales, source notes, rejected alternatives, grammatical explanations, and maintenance instructions may use the language needed to explain the controlled terminology. This does not exempt ordinary software documentation from STE-Lint. A compliant example must satisfy the constraints it claims to demonstrate; a non-STE example or counterexample must be identified as such. Do not implement this boundary with per-line lint suppressions.

## Evidence

`data/issue9-source.manifest.json` identifies the retained ASD-STE100 Issue 9 authority that supplies the general-dictionary versus subject-field terminology model. GitHub #57 is the owner-approved v3 admission/exclusion authority. `ste-lint-software-core-v3-grammar-review` records the explicit role/form/alias review. The last column gives an additional public terminology reference used in the review. Prior v1/v2 curated baselines are history, not admission authority.

Additional reference keys: `mdn-glossary` = MDN Web Docs Glossary; `google-developer-style` = Google Developer Documentation Style Guide; `google-api-reference-comments` = Google API reference comments guidance; `github-actions-concepts` = GitHub Actions workflows/actions concepts; `nist-csrc-glossary` = NIST CSRC Glossary; `opentelemetry-signals` = OpenTelemetry Signals; `postgresql-glossary` = PostgreSQL Documentation Glossary.

## Admitted terms

| Term | Role | Bounded concept | Why software-core needs it | Additional reference |
| --- | --- | --- | --- | --- |
| `API` | noun | A defined programmatic interface through which software systems or components communicate. | Keeps the established API concept distinct from a generic interface. General STE vocabulary does not preserve this bounded software distinction. | `google-api-reference-comments` |
| `application` | noun | A software program designed to perform one or more user or system functions. | Names the software-program concept rather than the ordinary act of applying something. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `client` | noun | Software that initiates communication with a server or service. | Names a stable software interaction role. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `command` | noun | A discrete instruction accepted by software or a command-line interface. | Names a software instruction object rather than a general request. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `configuration` | noun | Structured settings that control software behavior. | Names a stable software settings concept; casual shorthand such as config is not needed. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `database` | noun | An organized collection of data managed for software access. | Names the established database concept. General STE vocabulary does not preserve this bounded software distinction. | `postgresql-glossary` |
| `dependency` | noun | A software package, component, or resource required by other software. | Names a software prerequisite relationship used across build and package systems. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `directory` | noun | A file-system container for files or other directories. | Names the file-system container concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `endpoint` | noun | An addressable communication point of a software interface or service. | Names a stable API and service communication concept. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `environment` | noun | The runtime and configuration context in which software operates. | Names the software execution/configuration context. General STE vocabulary does not preserve this bounded software distinction. | `github-actions-concepts` |
| `file` | noun | A named data object stored by a file system. | Names the file-system object concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `identifier` | noun | A name or value used to distinguish a software entity. | Names the software identity token concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `interface` | noun | A defined boundary or contract through which software components interact. | Names the software interaction-contract concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `library` | noun | Reusable software intended for use by other software. | Names a stable software category distinct from an application. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `module` | noun | A named software unit that groups related code or behavior. | Names a stable software organizational unit. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `package` | noun | A distributable or managed unit of software. | Names the package-management/distribution unit. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `path` | noun | A representation of a location in a file system or software namespace. | Names the software/filesystem location representation. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `program` | noun | A set of software instructions designed for execution. | Names the executable software-instruction concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `runtime` | noun | The software or execution environment in which a program operates. | Names runtime concerns distinct from source/build concerns. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `schema` | noun | A formal definition of data structure and constraints. | Names the formal data-shape/constraint concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `server` | noun | Software that receives requests and provides a capability or resource. | Names the server interaction role. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `service` | noun | A software capability exposed through a defined interaction boundary. | Names a software service concept rather than ordinary service/help. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `version` | noun | An identified state of software or data with lifecycle or compatibility significance. | Names software/data version identity. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `CLI` | noun | A command-line interface that accepts textual commands. | Names the established command-line interface category. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `argument` | noun | A value supplied to a command, function, or program. | Preserves the programming distinction between a supplied argument and a declared parameter. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `binary` | noun | An executable or other machine-processable software file. | Names the software binary artifact concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `class` | noun | A software type that defines data or behavior for its instances. | Names the programming-language class construct. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `code` | noun | Instructions or declarations written in a programming language. | Distinguishes source/programming code from ordinary identification-code meanings. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `constant` | noun | A named software value intended not to change during its defined scope. | Names the programming constant/binding concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `function` | noun | A named callable unit of software behavior. | Names the callable-programming-unit sense rather than ordinary purpose/function. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `method` | noun | An operation associated with a software type or object. | Names the object/type-associated operation construct. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `object` | noun | A software entity represented according to a type or structure. | Names the software object construct rather than an ordinary physical object. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `parameter` | noun | A named input declared by a function, command, or interface. | Preserves the programming distinction between a declared parameter and a supplied argument. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `property` | noun | A named software value associated with an object or configuration. | Names the software member/property construct. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `string` | noun | A sequence of characters represented as a software value or type. | Names the programming string value/type. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `value` | noun | Software data represented by an expression, field, parameter, or variable. | Software values include non-quantitative typed data, so the technical sense is broader than ordinary quantitative value. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `variable` | noun | A named software binding or location that refers to a value. | Names the programming/configuration variable concept. General STE vocabulary does not preserve this bounded software distinction. | `github-actions-concepts` |
| `alias` | noun | An alternate software name that refers to an existing identity. | Names a stable software naming relationship. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `array` | noun | An ordered software collection whose values are addressed by position. | Names the fundamental array data structure. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `Boolean` | noun | A software value or type with the logical values true and false. | Names the logical Boolean value/type. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `byte` | noun | A unit of digital data conventionally made of eight bits. | Names the standard digital-data unit. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `field` | noun | A named member of a structured record, object, or message. | Names the structured-data field concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `namespace` | noun | A named software scope used to distinguish identifiers. | Names the identifier-scope concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `record` | noun | A structured collection of related named fields. | Names the structured-data noun sense, distinct from the general verb RECORD. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `table` | noun | A software data structure organized into rows and fields. | Names the tabular/database data structure. General STE vocabulary does not preserve this bounded software distinction. | `postgresql-glossary` |
| `cache` | noun | Stored data used to avoid repeating a software operation. | Names the caching mechanism while intentionally excluding the extra technical verb. General STE vocabulary does not preserve this bounded software distinction. | `github-actions-concepts` |
| `event` | noun | A represented occurrence that software can record, emit, or process. | Names the event-system concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `execution` | noun | One occurrence of running software instructions or an operation. | Names an execution instance rather than an abstract action. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `error` | noun | A software diagnostic or condition indicating that an operation did not complete as intended. | Names the software diagnostic/failure category. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `job` | noun | A bounded software work object submitted for execution by a scheduler, CI system, or worker. | Names a scheduler/CI/work-object concept rather than ordinary employment/work. General STE vocabulary does not preserve this bounded software distinction. | `github-actions-concepts` |
| `log` | noun | A recorded sequence of software events or messages. | Names the operational log object while preferring RECORD for the action. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `message` | noun | A discrete unit of information passed between software components. | Names the messaging/IPC unit concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `process` | noun | A running software instance managed by an operating system or runtime. | Narrows process to the OS/runtime instance and excludes vague verb use. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `request` | noun | A protocol or API message that asks software to perform an operation or return information. | Names the protocol/API request object while avoiding redundant technical-verb authority. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `response` | noun | Information returned by software in reply to a request. | Names the protocol/API response object. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `state` | noun | The formal represented condition of a software system or entity at a point in time. | Names formal represented system state rather than an ordinary condition. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `warning` | noun | A software diagnostic condition that can require attention without being a fatal error. | Names a software diagnostic severity/category. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `worker` | noun | A software execution component that performs assigned work. | Names the software worker component rather than a person. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `buffer` | noun | Temporary storage used while software processes or transfers data. | Names the temporary-storage mechanism while excluding the redundant verb. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `header` | noun | Structured metadata that precedes or accompanies a software message, request, or file. | Names the protocol/file header concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `index` | noun | A software data structure used to locate data efficiently. | Names the lookup structure while excluding the avoidable technical verb. General STE vocabulary does not preserve this bounded software distinction. | `postgresql-glossary` |
| `payload` | noun | Application data carried by a message, request, event, or protocol unit. | Names the carried application-data portion distinct from headers/metadata. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `protocol` | noun | Defined rules for software communication or interaction. | Names the communication protocol concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `query` | noun | A structured request for software data. | Names the structured data-request object while excluding redundant verb authority. General STE vocabulary does not preserve this bounded software distinction. | `postgresql-glossary` |
| `queue` | noun | An ordered collection of pending software work or messages. | Names the queue data/work structure while preferring a general verb for insertion. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `retry` | noun | A repeated attempt made after a software operation does not complete successfully. | Names the reliability mechanism while preferring TRY AGAIN for the action. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `session` | noun | A bounded period of related interaction maintained by software. | Names the stateful software-session concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `stream` | noun | A sequence of data processed or transferred over time. | Names the stream data concept while excluding unnecessary verb authority. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `thread` | noun | A sequence of execution scheduled within a software process. | Names the concurrency/runtime thread concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `timeout` | noun | A configured time limit after which software treats an operation as incomplete or failed. | Names the software timeout mechanism/condition. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `transaction` | noun | A bounded set of software operations treated as a unit for consistency or completion. | Names the database/state transaction concept. General STE vocabulary does not preserve this bounded software distinction. | `postgresql-glossary` |
| `artifact` | noun | A file or collection of files produced or retained by a software process. | Names a build/test/CI deliverable concept. General STE vocabulary does not preserve this bounded software distinction. | `github-actions-concepts` |
| `test case` | noun | Defined conditions and expected behavior used to verify software. | Names the specific test-case construct even though ordinary TEST is already general vocabulary. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `assertion` | noun | A condition that software or a test requires to be true. | Names the test/runtime assertion construct. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `benchmark` | noun | A defined measurement used to evaluate software performance. | Names the performance-measurement object while preferring MEASURE for the action. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `build` | noun | One software build result or one execution of the process that turns source inputs into software artifacts. | Names a software build object/execution while not granting the broad verb. General STE vocabulary does not preserve this bounded software distinction. | `github-actions-concepts` |
| `compiler` | noun | Software that translates source code into executable or machine-processable form. | Names the software that performs compilation. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `coverage` | noun | A measure of specified software behavior or code exercised by tests. | Names the software-testing coverage measure. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `deserialization` | noun | The conversion of serialized data into an in-memory software representation. | Names the exact data-representation conversion process. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `fixture` | noun | Controlled test data or setup used to establish repeatable test conditions. | Names the testing fixture construct. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `framework` | noun | Reusable software structure and conventions used to build applications or components. | Names a software category distinct from library/plugin/application. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `manifest` | noun | A structured file or record that declares software contents, identity, configuration, or dependencies. | Names the software manifest concept across packages/builds/deployments. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `migration` | noun | A controlled software change that moves data, schema, configuration, or state between representations. | Names the controlled representation-change concept. General STE vocabulary does not preserve this bounded software distinction. | `postgresql-glossary` |
| `parser` | noun | Software that analyzes structured input according to syntax. | Names the software that performs syntax analysis. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `persistence` | noun | The software capability to retain data or state beyond the operation that created it. | Names the software capability while avoiding developer-shorthand verb persist. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `plugin` | noun | A software component that extends an application or framework through a defined extension mechanism. | Names the defined extension-mechanism component category. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `regression` | noun | Previously correct software behavior that becomes incorrect after a change. | Names the testing/software regression concept rather than general backward movement. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `release` | noun | An identified software version made available for use. | Names the software release object while preferring general vocabulary for the act where possible. General STE vocabulary does not preserve this bounded software distinction. | `github-actions-concepts` |
| `serialization` | noun | The conversion of an in-memory software representation into a storage or transfer representation. | Names the exact data-representation conversion process. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `authentication` | noun | The process of establishing the identity associated with a user, request, or system. | Names a specific security process distinct from authorization. General STE vocabulary does not preserve this bounded software distinction. | `nist-csrc-glossary` |
| `authorization` | noun | The process of determining whether an identity may perform an operation or access a resource. | Names a specific security process distinct from authentication. General STE vocabulary does not preserve this bounded software distinction. | `nist-csrc-glossary` |
| `credential` | noun | Information used to establish or prove a software identity. | Names the security identity-evidence concept. General STE vocabulary does not preserve this bounded software distinction. | `nist-csrc-glossary` |
| `identity` | noun | Information by which software distinguishes a represented subject or entity. | Names the software/security identity concept. General STE vocabulary does not preserve this bounded software distinction. | `nist-csrc-glossary` |
| `metadata` | noun | Data that describes other software data, artifacts, resources, or entities. | Names the descriptive-data concept. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `metric` | noun | A measured value that describes software behavior, performance, or operation. | Names the observability measurement concept. General STE vocabulary does not preserve this bounded software distinction. | `opentelemetry-signals` |
| `permission` | noun | An authorization grant or rule that allows an identity to perform an operation. | Names the software access-control grant/rule concept. General STE vocabulary does not preserve this bounded software distinction. | `nist-csrc-glossary` |
| `secret` | noun | Sensitive software configuration or authentication data that must be protected from unauthorized disclosure. | Names the protected software-secret concept. General STE vocabulary does not preserve this bounded software distinction. | `github-actions-concepts` |
| `telemetry` | noun | Operational data collected from software for observation or analysis. | Names the observability data category. General STE vocabulary does not preserve this bounded software distinction. | `opentelemetry-signals` |
| `trace` | noun | A recorded path of software execution or distributed activity. | Names the observability trace object while preferring RECORD for the act. General STE vocabulary does not preserve this bounded software distinction. | `opentelemetry-signals` |
| `compile` | verb | To translate source code into executable or machine-processable form. | Names the specific compilation process; a general verb would lose the transformation. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `deploy` | verb | To make software available in a target execution environment. | Names the software deployment process rather than a physical deployment sense. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `deserialize` | verb | To convert serialized data into an in-memory software representation. | Names the exact conversion operation. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `execute` | verb | To run software instructions or an operation. | Names execution of code/instructions; DO does not preserve this technical distinction. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `export` | verb | To move or expose software data across an outward system or format boundary. | Names a defined outward data/software boundary operation. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `import` | verb | To bring software data or code across an inward system or format boundary. | Names the corresponding inward data/software boundary operation. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `install` | verb | To place and configure software so that it can be used in a target environment. | Names a specified computer-system installation process. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |
| `load` | verb | To bring software or data into an active processing context. | Names a specified computer-system loading process. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `parse` | verb | To analyze structured input according to defined syntax. | Names syntax analysis; a generic examine/check verb would lose the syntax constraint. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `serialize` | verb | To convert an in-memory software representation into a form for storage or transfer. | Names the exact representation-conversion operation. General STE vocabulary does not preserve this bounded software distinction. | `mdn-glossary` |
| `validate` | verb | To check software or data against explicit requirements or constraints. | Names the specified software/data validation process rather than generic checking. General STE vocabulary does not preserve this bounded software distinction. | `google-developer-style` |

## Explicit exclusions

These are durable negative decisions. Corpus review must not silently re-admit them because they are frequent in developer prose.

| Term | Why excluded | Preferred treatment |
| --- | --- | --- |
| `component` | ASD-STE100 already supplies a general COMPONENT concept adequate for a part or unit; the software-core definition did not establish a narrower concept. | Use the general word when applicable, or name the more specific software concept such as module, service, library, or package. |
| `data` | General DATA already covers software data; no distinct software subject-field concept is established by the profile entry. | Use the ASD-STE100 dictionary authority for ordinary data. |
| `implementation` | The term is abstract and context-dependent rather than one stable software concept. | Name the actual code, program, module, library, service, or other governed technical object. |
| `input` | General INPUT already covers data supplied to a system. | Use the ASD-STE100 dictionary authority unless a project defines a narrower named input object. |
| `output` | General OUTPUT already covers data produced by a system. | Use the ASD-STE100 dictionary authority unless a project defines a narrower named output object. |
| `status` | The term does not identify one stable cross-project software concept; its meaning depends on a particular lifecycle or data model. | Use ordinary STE where applicable or define the specific project status model locally. |
| `task` | General TASK already covers assigned or bounded work; framework-specific task objects are not one universal software concept. | Use general TASK, or define a framework/project task object locally when needed. |
| `test` | General TEST already covers a procedure used to establish correct performance or function. | Use the ASD-STE100 dictionary authority; software-core retains the more specific technical noun test case. |
| `compatibility` | This is an abstract relationship whose meaning depends on the versions, interfaces, or environments being compared. | State the concrete compatibility condition or define a narrower project concept. |
| `entity` | The technical meaning varies across domain models, ORMs, databases, and project architectures. | Define the project/domain entity concept locally when authoritative evidence establishes it. |
| `integration` | The word can mean a connector, combined system, test class, deployment relationship, or activity, so it does not identify one bounded software concept. | Name the actual interface, service, test, connector, or other specific concept. |
| `result` | General RESULT already covers what occurs from an action; the software-core entry did not materially narrow that meaning. | Use the ASD-STE100 dictionary authority or a more specific software term when one exists. |
| `configure` | The action is normally expressible with an approved general verb such as set plus the configuration or option being changed. | Prefer the approved general verb and the governed technical noun configuration. |
| `verify` | The proposed meaning was generic rather than a distinct software process. | Use the approved STE alternative for ordinary verification; use validate when the specified software/data validation process is intended. |
| `persist` | This is developer shorthand for retaining data or state, and the action is normally expressible with approved store. | Use STORE for the action; retain persistence as the technical noun for the software capability. |

## Role and alias decisions

Noun-only: `cache`, `log`, `process`, `request`, `buffer`, `index`, `query`, `queue`, `retry`, `stream`, `benchmark`, `build`, `release`, `trace`.

Verb-only: `export`, `import`. Other admitted technical verbs are bounded to the processes recorded above.

`config` is not an alias for `configuration`. Established abbreviations/designations are governed only when explicit; current examples include `API`, `CLI`, and `ID`/`IDs`.

All grammatical forms are explicit. STE-Lint does not infer terminology authority through stemming or generated morphology.

## Future admission procedure

1. Classify the spelling against the verified ASD-STE100 runtime first.
2. Establish a specified software subject-field concept/process, not merely common developer usage.
3. Establish cross-codebase stability.
4. For a verb, establish that approved general vocabulary cannot communicate the operation accurately.
5. Record independent definition evidence and only the roles/forms/aliases/status that evidence supports.
6. Check the exclusions above before proposing an addition.
7. Add regressions for the technical sense and relevant ordinary-language false-positive cases.

Reviewed for profile version 3 on 2026-08-17.
