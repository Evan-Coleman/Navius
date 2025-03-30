# navius-test API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 97%  
**Status:** ✅ Good

## Dependencies

- async-trait
- thiserror
- tokio
- serde
- serde_json
- uuid
- tracing
- mockall
- tempfile
- futures
- once_cell

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| FixtureState | fixture.rs | 11 | ⚠️ Partial |
| FixtureConfig | fixture.rs | 33 | ⚠️ Partial |
| TestFixture | fixture.rs | 59 | ✅ Complete |
| TestFixtureBuilder | fixture.rs | 216 | ⚠️ Partial |
| TempDirectory | fixture.rs | 273 | ❌ Missing |
| TestHarness | harness.rs | 10 | ⚠️ Partial |
| TestHarnessBuilder | harness.rs | 123 | ⚠️ Partial |
| MockRegistryState | mock.rs | 8 | ⚠️ Partial |
| Expectation | mock.rs | 32 | ⚠️ Partial |
| MockRegistry | mock.rs | 91 | ⚠️ Partial |
| ExpectationBuilder<'a> | mock.rs | 317 | ⚠️ Partial |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| TestError | error.rs | 9 | ✅ Complete |

## Public Traits

| Name | File | Line | Documentation |
|------|------|------|---------------|
| IntoTestError | error.rs | 103 | ⚠️ Partial |
| Resource | fixture.rs | 50 | ⚠️ Partial |
| MockBuilder<T> | mock.rs | 295 | ⚠️ Partial |
| RegisterMock<Interface, | mock.rs | 301 | ✅ Complete |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| setup_error<S: | error.rs | 57 | ⚠️ Partial |
| teardown_error<S: | error.rs | 62 | ⚠️ Partial |
| missing_component<S: | error.rs | 67 | ⚠️ Partial |
| mock_not_registered<S: | error.rs | 72 | ⚠️ Partial |
| invalid_configuration<S: | error.rs | 77 | ⚠️ Partial |
| resource_allocation_error<S: | error.rs | 82 | ⚠️ Partial |
| assertion_error<S: | error.rs | 87 | ⚠️ Partial |
| external_error<S: | error.rs | 92 | ⚠️ Partial |
| unknown<S: | error.rs | 97 | ⚠️ Partial |
| new(config: | fixture.rs | 22 | ✅ Complete |
| new() | fixture.rs | 66 | ✅ Complete |
| register<T: | fixture.rs | 71 | ⚠️ Partial |
| register_resource<R: | fixture.rs | 91 | ⚠️ Partial |
| get<T: | fixture.rs | 110 | ⚠️ Partial |
| get_as<T: | fixture.rs | 134 | ⚠️ Partial |
| has<T: | fixture.rs | 154 | ⚠️ Partial |
| cleanup(&self) | fixture.rs | 164 | ⚠️ Partial |
| config(&self) | fixture.rs | 192 | ⚠️ Partial |
| new() | fixture.rs | 222 | ⚠️ Partial |
| with_auto_cleanup(mut | fixture.rs | 229 | ⚠️ Partial |
| with_verbose_logging(mut | fixture.rs | 235 | ⚠️ Partial |
| build(self) | fixture.rs | 241 | ⚠️ Partial |
| with_component<T: | fixture.rs | 248 | ⚠️ Partial |
| with_resource<R: | fixture.rs | 257 | ⚠️ Partial |
| new(path: | fixture.rs | 278 | ❌ Missing |
| new() | harness.rs | 23 | ✅ Complete |
| with_runtime() | harness.rs | 32 | ⚠️ Partial |
| fixture(&self) | harness.rs | 48 | ⚠️ Partial |
| mock_registry(&self) | harness.rs | 53 | ⚠️ Partial |
| run<F, | harness.rs | 58 | ⚠️ Partial |
| run_async<F, | harness.rs | 73 | ⚠️ Partial |
| cleanup(&self) | harness.rs | 99 | ⚠️ Partial |
| new() | harness.rs | 133 | ✅ Complete |
| with_runtime(mut | harness.rs | 141 | ⚠️ Partial |
| with_fixture(mut | harness.rs | 147 | ⚠️ Partial |
| build(self) | harness.rs | 153 | ⚠️ Partial |
| new() | mock.rs | 21 | ✅ Complete |
| new(method_name: | mock.rs | 51 | ✅ Complete |
| with_min_calls(mut | mock.rs | 62 | ⚠️ Partial |
| with_max_calls(mut | mock.rs | 68 | ⚠️ Partial |
| record_call(&mut | mock.rs | 74 | ⚠️ Partial |
| is_satisfied(&self) | mock.rs | 83 | ⚠️ Partial |
| new() | mock.rs | 98 | ✅ Complete |
| register<Interface, | mock.rs | 105 | ⚠️ Partial |
| get<Interface, | mock.rs | 129 | ⚠️ Partial |
| has<Interface>(&self) | mock.rs | 149 | ⚠️ Partial |
| remove<Interface>(&self) | mock.rs | 169 | ⚠️ Partial |
| expect<Interface>(&self, | mock.rs | 186 | ⚠️ Partial |
| record_call<Interface>(&self, | mock.rs | 206 | ⚠️ Partial |
| verify(&self) | mock.rs | 230 | ⚠️ Partial |
| reset(&self) | mock.rs | 262 | ⚠️ Partial |
| new<Interface>(registry: | mock.rs | 336 | ✅ Complete |
| min_calls(mut | mock.rs | 350 | ⚠️ Partial |
| max_calls(mut | mock.rs | 356 | ⚠️ Partial |
| times(mut | mock.rs | 362 | ⚠️ Partial |
| build(self) | mock.rs | 369 | ⚠️ Partial |

