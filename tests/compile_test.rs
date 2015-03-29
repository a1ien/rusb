#![feature(core,collections)]

extern crate regex;

use regex::Regex;

use std::process::{Command,Output};
use std::str::FromStr;


#[derive(Debug)]
struct CompileResult {
  success: bool,
  errors: Vec<CompileError>
}

impl CompileResult {
  fn new(output: Output) -> Self {
    let re = Regex::new(r"(?m)^(?P<file>.*):(?P<line>\d+):(?:\d+): (?:\d+):(?:\d+) error: (?P<message>.*)$").unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    let mut errors = Vec::<CompileError>::new();

    for capture in re.captures_iter(stderr.as_slice()) {
      let file = capture.name("file").unwrap();
      let line: usize = FromStr::from_str(capture.name("line").unwrap()).unwrap();
      let message = capture.name("message").unwrap();

      errors.push(CompileError::new(file, line, message));
    }

    CompileResult {
      success: output.status.success(),
      errors: errors
    }
  }

  fn success(&self) -> bool {
    self.errors.len() == 0
  }

  fn has_error(&self, message: &str, line: usize) -> bool {
    for error in self.errors.iter() {
      if error.line == line && error.message.find(message).is_some() {
        return true
      }
    }

    false
  }
}


#[derive(Debug)]
struct CompileError {
  file: String,
  line: usize,
  message: String
}

impl CompileError {
  fn new(file: &str, line: usize, message: &str) -> Self {
    CompileError {
      file: String::from_str(file),
      line: line,
      message: String::from_str(message)
    }
  }
}


const ERR_LIFETIME: &'static str = "does not live long enough";


macro_rules! assert_compile_success {
  ($file:expr) => ({
    let result = compile($file);
    assert!(result.success(), "expected compilation of {:?} to succeed, got: {:?}", $file, result);
    result
  })
}

macro_rules! assert_compile_fail {
  ($file:expr) => ({
    let result = compile($file);
    assert!(!result.success(), "expected compilation of {:?} to fail", $file);
    result
  });
  ($file:expr,$msg:expr,$line:expr) => ({
    let result = compile($file);
    assert!(!result.success(), "expected compilation of {:?} to fail", $file);
    assert_compile_error!(result, $msg, $line);
    result
  });
}

macro_rules! assert_compile_error {
  ($result:expr,$msg:expr,$line:expr) => ({
    if !$result.has_error($msg, $line) {
      panic!("assertion failed: expected error {:?} on line {}, found {:?}", $msg, $line, $result);
    }
  });
}


fn compile(file: &str) -> CompileResult {
  CompileResult::new(
    Command::new("rustc").
      arg("-L").arg("target/debug").
      arg("-L").arg("target/debug/deps").
      arg(format!("tests/compile-tests/{}", file).as_slice()).
      arg("--out-dir").arg("target/debug/compile-tests").
      output().unwrap()
  )
}


#[test]
fn it_should_not_compile_device_list_that_outlives_context() {
  assert_compile_fail!("device_list_outlives_context.rs", ERR_LIFETIME, 6);
}

#[test]
fn it_should_not_compile_device_ref_that_outlives_context() {
  assert_compile_fail!("device_ref_outlives_context.rs", ERR_LIFETIME, 6);
}

#[test]
fn it_should_compile_device_ref_that_outlives_device_list() {
  assert_compile_success!("device_ref_outlives_device_list.rs");
}

#[test]
fn it_should_not_compile_device_iterator_that_outlives_device_list() {
  assert_compile_fail!("device_iterator_outlives_device_list.rs", ERR_LIFETIME, 8);
}

#[test]
fn it_should_not_compile_device_handle_that_outlives_context() {
  assert_compile_fail!("device_handle_outlives_context.rs", ERR_LIFETIME, 6);
}

#[test]
fn it_should_compile_device_handle_that_outlives_device_ref() {
  assert_compile_success!("device_handle_outlives_device_ref.rs");
}

#[test]
fn it_should_not_compile_interface_handle_that_outlives_device_handle() {
  assert_compile_fail!("interface_handle_outlives_device_handle.rs", ERR_LIFETIME, 9);
}
