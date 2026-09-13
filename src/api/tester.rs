use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub request: super::client::APIRequest,
    pub assertions: Vec<Assertion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Assertion {
    StatusCode(u16),
    HeaderEquals(String, String),
    BodyContains(String),
    BodyEquals(String),
    ResponseTimeLessThan(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub assertions: Vec<AssertionResult>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    pub assertion: Assertion,
    pub passed: bool,
    pub message: String,
}

pub struct APITester {
    test_cases: Vec<TestCase>,
}

impl APITester {
    pub fn new() -> Result<Self> {
        Ok(Self {
            test_cases: Vec::new(),
        })
    }

    pub fn add_test_case(&mut self, test_case: TestCase) {
        self.test_cases.push(test_case);
    }

    pub async fn run_test(&self, test_case: &TestCase) -> TestResult {
        let start = std::time::Instant::now();
        
        let client = match super::client::APIClient::new() {
            Ok(c) => c,
            Err(e) => {
                return TestResult {
                    name: test_case.name.clone(),
                    passed: false,
                    assertions: vec![AssertionResult {
                        assertion: Assertion::StatusCode(200),
                        passed: false,
                        message: format!("Failed to create client: {}", e),
                    }],
                    duration_ms: 0,
                };
            }
        };
        
        let response = client.execute(
            &super::client::APIRequest {
                method: test_case.request.method.clone(),
                url: test_case.request.url.clone(),
                headers: test_case.request.headers.clone(),
                body: test_case.request.body.clone(),
                timeout: test_case.request.timeout,
                follow_redirects: true,
            },
        ).await;
        
        let duration = start.elapsed().as_millis() as u64;
        
        match response {
            Ok(resp) => {
                let mut results = Vec::new();
                let mut all_passed = true;
                
                for assertion in &test_case.assertions {
                    let result = self.check_assertion(assertion, &resp);
                    if !result.passed {
                        all_passed = false;
                    }
                    results.push(result);
                }
                
                TestResult {
                    name: test_case.name.clone(),
                    passed: all_passed,
                    assertions: results,
                    duration_ms: duration,
                }
            }
            Err(e) => TestResult {
                name: test_case.name.clone(),
                passed: false,
                assertions: vec![AssertionResult {
                    assertion: Assertion::StatusCode(200),
                    passed: false,
                    message: format!("Request failed: {}", e),
                }],
                duration_ms: duration,
            },
        }
    }

    fn check_assertion(&self, assertion: &Assertion, response: &super::client::APIResponse) -> AssertionResult {
        match assertion {
            Assertion::StatusCode(expected) => {
                let passed = response.status == *expected;
                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message: if passed {
                        format!("Status code matches: {}", expected)
                    } else {
                        format!("Expected status {}, got {}", expected, response.status)
                    },
                }
            }
            Assertion::HeaderEquals(key, expected) => {
                let actual = response.headers.get(key).map(|s| s.as_str()).unwrap_or("");
                let passed = actual == expected;
                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message: if passed {
                        format!("Header {} matches", key)
                    } else {
                        format!("Header {} expected '{}', got '{}'", key, expected, actual)
                    },
                }
            }
            Assertion::BodyContains(text) => {
                let passed = response.body.contains(text);
                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message: if passed {
                        format!("Body contains '{}'", text)
                    } else {
                        format!("Body does not contain '{}'", text)
                    },
                }
            }
            Assertion::BodyEquals(expected) => {
                let passed = response.body == *expected;
                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message: if passed {
                        "Body matches".to_string()
                    } else {
                        "Body does not match expected value".to_string()
                    },
                }
            }
            Assertion::ResponseTimeLessThan(max_ms) => {
                let passed = response.duration_ms <= *max_ms;
                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message: if passed {
                        format!("Response time {}ms < {}ms", response.duration_ms, max_ms)
                    } else {
                        format!("Response time {}ms >= {}ms", response.duration_ms, max_ms)
                    },
                }
            }
        }
    }

    pub async fn run_all(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        for test_case in &self.test_cases {
            println!("Running test: {}", test_case.name);
            let result = self.run_test(test_case).await;
            results.push(result);
        }
        
        results
    }

    pub fn print_results(&self, results: &[TestResult]) {
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;
        
        println!("\n📊 Test Results:");
        println!("════════════════════════════════════════════════════════════════════════════════");
        
        for result in results {
            let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
            println!("{} {} ({}ms)", status, result.name, result.duration_ms);
            
            for assertion_result in &result.assertions {
                if !assertion_result.passed {
                    println!("   └─ {}", assertion_result.message);
                }
            }
        }
        
        println!("════════════════════════════════════════════════════════════════════════════════");
        println!("Total: {} | Passed: {} | Failed: {}", results.len(), passed, failed);
    }
}
