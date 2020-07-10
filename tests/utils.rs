use std::{ffi::OsString, fs::DirEntry, path::Path};

/// Manages the setting up and running of expansion tests using macrotest
///
/// Expansion test live in a home directory. This directory has a single
/// testing sub-directory that is used during the test. Temporary testing
/// files are put in the testing directory before each test but not removed
/// after. (They may be deleted before each test, through)
///
/// The tester is configured to generate files in the testing directory from
/// files in source directories (sub-directories of the home).
/// Various rules can be configured, e.g. a simple copy of files, or duplicating
/// the source files a number of times in the testing directory with various
/// names.
pub struct ExpansionTester<'a>
{
	/// The home directory for the tests
	dir: &'a str,
	/// The subdirectory (of the home) where test files may be put
	testing_dir: &'a str,
	/// Source sub-directory, and how ea
	source_dirs: Vec<(&'a str, Box<dyn Fn(&DirEntry, &dyn AsRef<Path>)>)>,
}

impl<'a> ExpansionTester<'a>
{
	/// Construct a new tester with a home directory and a testing subdirectory.
	pub fn new(home_dir: &'a str, testing_dir: &'a str) -> Self
	{
		Self {
			dir: home_dir,
			testing_dir,
			source_dirs: Vec::new(),
		}
	}

	/// Add a source directory under the home directory,
	/// with an action that produces files in the testing directory
	/// based on each file in the source directory.
	pub fn add_source_dir(&mut self, dir: &'a str, action: Box<dyn Fn(&DirEntry, &dyn AsRef<Path>)>)
	{
		self.source_dirs.push((dir, action));
	}

	/// Executes the tests including first setting up the testing directory.
	pub fn execute_tests(&self)
	{
		// Remove old test files
		let testing_dir = self.dir.to_owned() + "/" + self.testing_dir;
		let _ = std::fs::remove_dir_all(&testing_dir);

		// Recreate testing dir
		std::fs::create_dir_all(&testing_dir).unwrap();

		// For each source dir, execute action of each file
		for (source_dir, action) in self.source_dirs.iter()
		{
			let source_dir_path = self.dir.to_owned() + "/" + source_dir;
			if let Ok(files) = std::fs::read_dir(&source_dir_path)
			{
				for file in files
				{
					if let Ok(file) = file
					{
						action(&file, &testing_dir);
					}
					else
					{
						panic!(format!("Error accessing source file: {:?}", file))
					}
				}
			}
		}

		macrotest::expand_without_refresh(testing_dir + "/*.rs");
	}

	/// Generates an action that simply copies the file given to the testing
	/// directory.
	pub fn copy() -> Box<dyn Fn(&DirEntry, &dyn AsRef<Path>)>
	{
		Box::new(|file, destination| {
			let mut destination_file = destination.as_ref().to_path_buf();
			destination_file.push(file.file_name());
			std::fs::copy(&file.path(), &destination_file).unwrap();
		})
	}

	/// Generates an action that duplicates the file given a number of times.
	/// The given function mus take the original file name and produce
	/// all the file names that must be duplicated in the testing directory.
	pub fn duplicate(
		duplicates: fn(OsString) -> Vec<OsString>,
	) -> Box<dyn Fn(&DirEntry, &dyn AsRef<Path>)>
	{
		Box::new(move |file, destination| {
			for duplicate in duplicates(file.file_name()).into_iter()
			{
				let mut destination_file = destination.as_ref().to_path_buf();
				destination_file.push(duplicate);
				std::fs::copy(&file.path(), &destination_file).unwrap();
			}
		})
	}
}