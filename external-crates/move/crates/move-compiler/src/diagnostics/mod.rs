// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod codes;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    io::Write,
    iter::FromIterator,
    ops::Range,
    path::PathBuf,
    sync::Arc,
};

use codespan_reporting::{
    self as csr,
    files::SimpleFiles,
    term::{
        emit,
        termcolor::{Buffer, ColorChoice, StandardStream, WriteColor},
        Config,
    },
};
use csr::files::Files;
use move_command_line_common::{env::read_env_var, files::FileHash};
use move_ir_types::location::*;
use move_symbol_pool::Symbol;

use self::codes::UnusedItem;
use crate::{
    command_line::COLOR_MODE_ENV_VAR,
    diagnostics::codes::{
        Category, DiagnosticCode, DiagnosticInfo, ExternalPrefix, Severity, WarningFilter,
        WellKnownFilterName,
    },
    shared::{
        ast_debug::AstDebug, known_attributes, FILTER_UNUSED_CONST, FILTER_UNUSED_FUNCTION,
        FILTER_UNUSED_MUT_PARAM, FILTER_UNUSED_MUT_REF, FILTER_UNUSED_STRUCT_FIELD,
        FILTER_UNUSED_TYPE_PARAMETER,
    },
};

//**************************************************************************************************
// Types
//**************************************************************************************************

pub type FileId = usize;
pub type FileName = Symbol;

pub type FilesSourceText = HashMap<FileHash, (FileName, Arc<str>)>;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
#[must_use]
pub struct Diagnostic {
    info: DiagnosticInfo,
    primary_label: (Loc, String),
    secondary_labels: Vec<(Loc, String)>,
    notes: Vec<String>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct Diagnostics(Option<Diagnostics_>);

#[derive(PartialEq, Eq, Hash, Clone, Debug, Default)]
struct Diagnostics_ {
    diagnostics: Vec<Diagnostic>,
    // diagnostics filtered in source code
    filtered_source_diagnostics: Vec<Diagnostic>,
    severity_count: BTreeMap<Severity, usize>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
/// Used to filter out diagnostics, specifically used for warning suppression
pub struct WarningFilters {
    filters: BTreeMap<ExternalPrefix, UnprefixedWarningFilters>,
    for_dependency: bool, // if false, the filters are used for source code
}

#[derive(PartialEq, Eq, Clone, Debug)]
/// Filters split by category and code
enum UnprefixedWarningFilters {
    /// Remove all warnings
    All,
    Specified {
        /// Remove all diags of this category with optional known name
        categories: BTreeMap<u8, Option<WellKnownFilterName>>,
        /// Remove specific diags with optional known filter name
        codes: BTreeMap<(u8, u8), Option<WellKnownFilterName>>,
    },
    /// No filter
    Empty,
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
enum MigrationChange {
    AddMut,
    AddPublic,
    Backquote(String),
    AddGlobalQual,
    RemoveFriend,
    MakePubPackage,
    AddressRemove,
    AddressAdd(String),
}

// All of the migration changes
pub struct Migration {
    mapped_files: MappedFiles,
    changes: BTreeMap<FileId, Vec<(ByteSpan, MigrationChange)>>,
}

/// A mapping from file ids to file contents along with the mapping of filehash
/// to fileID.
pub struct MappedFiles {
    files: SimpleFiles<Symbol, Arc<str>>,
    file_mapping: HashMap<FileHash, FileId>,
}

/// A file, and the line:column start, and line:column end that corresponds to a
/// `Loc`
#[allow(dead_code)]
pub struct FileLineColSpan {
    pub file_id: FileId,
    pub start: LineColLocation,
    pub end: LineColLocation,
}

/// A line and column location in a file
pub struct LineColLocation {
    pub line: usize,
    pub column: usize,
    pub byte: usize,
}

/// A file, and the line:column start, and line:column end that corresponds to a
/// `Loc`
pub struct FileByteSpan {
    file_id: FileId,
    byte_span: ByteSpan,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteSpan {
    start: usize,
    end: usize,
}

impl MappedFiles {
    pub fn new(files: FilesSourceText) -> Self {
        let mut simple_files = SimpleFiles::new();
        let mut file_mapping = HashMap::new();
        for (fhash, (fname, source)) in files {
            let id = simple_files.add(fname, source);
            file_mapping.insert(fhash, id);
        }
        Self {
            files: simple_files,
            file_mapping,
        }
    }

    pub fn empty() -> Self {
        Self {
            files: SimpleFiles::new(),
            file_mapping: HashMap::new(),
        }
    }

    pub fn file_hash_to_file_id(&self, fhash: &FileHash) -> Option<FileId> {
        self.file_mapping.get(fhash).copied()
    }

    pub fn add(&mut self, fhash: FileHash, fname: FileName, source: Arc<str>) {
        let id = self.files.add(fname, source);
        self.file_mapping.insert(fhash, id);
    }

    #[allow(dead_code)]
    pub fn location(&self, loc: Loc) -> FileLineColSpan {
        let start_loc = loc.start() as usize;
        let end_loc = loc.end() as usize;
        let file_id = *self.file_mapping.get(&loc.file_hash()).unwrap();
        let start_file_loc = self.files.location(file_id, start_loc).unwrap();
        let end_file_loc = self.files.location(file_id, end_loc).unwrap();
        FileLineColSpan {
            file_id,
            start: LineColLocation {
                line: start_file_loc.line_number,
                column: start_file_loc.column_number - 1,
                byte: start_loc,
            },
            end: LineColLocation {
                line: end_file_loc.line_number,
                column: end_file_loc.column_number - 1,
                byte: end_loc,
            },
        }
    }

    pub fn byte_location(&self, loc: Loc) -> FileByteSpan {
        let start = loc.start() as usize;
        let end = loc.end() as usize;
        let file_id = *self.file_mapping.get(&loc.file_hash()).unwrap();
        FileByteSpan {
            byte_span: ByteSpan { start, end },
            file_id,
        }
    }
}

//**************************************************************************************************
// Diagnostic Reporting
//**************************************************************************************************

pub fn report_diagnostics(files: &FilesSourceText, diags: Diagnostics) -> ! {
    let should_exit = true;
    report_diagnostics_impl(files, diags, should_exit);
    std::process::exit(1)
}

pub fn report_warnings(files: &FilesSourceText, warnings: Diagnostics) {
    if warnings.is_empty() {
        return;
    }
    debug_assert!(warnings.max_severity().unwrap() == Severity::Warning);
    report_diagnostics_impl(files, warnings, false)
}

fn report_diagnostics_impl(files: &FilesSourceText, diags: Diagnostics, should_exit: bool) {
    let color_choice = env_color();
    let mut writer = StandardStream::stderr(color_choice);
    output_diagnostics(&mut writer, files, diags);
    if should_exit {
        std::process::exit(1);
    }
}

pub fn unwrap_or_report_pass_diagnostics<T, Pass>(
    files: &FilesSourceText,
    res: Result<T, (Pass, Diagnostics)>,
) -> T {
    match res {
        Ok(t) => t,
        Err((_pass, diags)) => {
            assert!(!diags.is_empty());
            report_diagnostics(files, diags)
        }
    }
}

pub fn unwrap_or_report_diagnostics<T>(files: &FilesSourceText, res: Result<T, Diagnostics>) -> T {
    match res {
        Ok(t) => t,
        Err(diags) => {
            assert!(!diags.is_empty());
            report_diagnostics(files, diags)
        }
    }
}

pub fn report_diagnostics_to_buffer_with_env_color(
    files: &FilesSourceText,
    diags: Diagnostics,
) -> Vec<u8> {
    let ansi_color = match env_color() {
        ColorChoice::Always | ColorChoice::AlwaysAnsi | ColorChoice::Auto => true,
        ColorChoice::Never => false,
    };
    report_diagnostics_to_buffer(files, diags, ansi_color)
}

pub fn report_diagnostics_to_buffer(
    files: &FilesSourceText,
    diags: Diagnostics,
    ansi_color: bool,
) -> Vec<u8> {
    let mut writer = if ansi_color {
        Buffer::ansi()
    } else {
        Buffer::no_color()
    };
    output_diagnostics(&mut writer, files, diags);
    writer.into_inner()
}

fn env_color() -> ColorChoice {
    match read_env_var(COLOR_MODE_ENV_VAR).as_str() {
        "NONE" => ColorChoice::Never,
        "ANSI" => ColorChoice::AlwaysAnsi,
        "ALWAYS" => ColorChoice::Always,
        _ => ColorChoice::Auto,
    }
}

fn output_diagnostics<W: WriteColor>(
    writer: &mut W,
    sources: &FilesSourceText,
    diags: Diagnostics,
) {
    let mapping = MappedFiles::new(sources.clone());
    render_diagnostics(writer, mapping, diags);
}

fn render_diagnostics(writer: &mut dyn WriteColor, mapping: MappedFiles, diags: Diagnostics) {
    let Diagnostics(Some(mut diags)) = diags else {
        return;
    };

    // Do not render / report migration diagnostics.
    diags.diagnostics.retain(|diag| !diag.is_migration());

    diags.diagnostics.sort_by(|e1, e2| {
        let loc1: &Loc = &e1.primary_label.0;
        let loc2: &Loc = &e2.primary_label.0;
        loc1.cmp(loc2)
    });
    let mut seen: HashSet<Diagnostic> = HashSet::new();
    for diag in diags.diagnostics {
        if seen.contains(&diag) {
            continue;
        }
        seen.insert(diag.clone());
        let rendered = render_diagnostic(&mapping, diag);
        emit(writer, &Config::default(), &mapping.files, &rendered).unwrap()
    }
}

fn convert_loc(mapped_files: &MappedFiles, loc: Loc) -> (FileId, Range<usize>) {
    let fname = loc.file_hash();
    let id = mapped_files.file_hash_to_file_id(&fname).unwrap();
    let range = loc.usize_range();
    (id, range)
}

fn render_diagnostic(
    mapped_files: &MappedFiles,
    diag: Diagnostic,
) -> csr::diagnostic::Diagnostic<FileId> {
    use csr::diagnostic::{Label, LabelStyle};
    let mk_lbl = |style: LabelStyle, msg: (Loc, String)| -> Label<FileId> {
        let (id, range) = convert_loc(mapped_files, msg.0);
        csr::diagnostic::Label::new(style, id, range).with_message(msg.1)
    };
    let Diagnostic {
        info,
        primary_label,
        secondary_labels,
        notes,
    } = diag;
    let mut diag = csr::diagnostic::Diagnostic::new(info.severity().into_codespan_severity());
    let (code, message) = info.render();
    diag = diag.with_code(code);
    diag = diag.with_message(message.to_string());
    diag = diag.with_labels(vec![mk_lbl(LabelStyle::Primary, primary_label)]);
    diag = diag.with_labels(
        secondary_labels
            .into_iter()
            .map(|msg| mk_lbl(LabelStyle::Secondary, msg))
            .collect(),
    );
    diag = diag.with_notes(notes);
    diag
}

//**************************************************************************************************
// Migration Diff Reporting
//**************************************************************************************************

pub fn generate_migration_diff(
    files: &FilesSourceText,
    diags: &Diagnostics,
) -> Option<(Migration, /* Migration errors */ Diagnostics)> {
    match diags {
        Diagnostics(Some(inner)) => {
            let migration_diags = inner
                .diagnostics
                .iter()
                .filter(|diag| diag.is_migration())
                .cloned()
                .collect::<Vec<_>>();
            if migration_diags.is_empty() {
                return None;
            }
            Some(Migration::new(files.clone(), migration_diags))
        }
        _ => None,
    }
}

// Used in test harness for unit testing
pub fn report_migration_to_buffer(files: &FilesSourceText, diags: Diagnostics) -> Vec<u8> {
    let mut writer = Buffer::no_color();
    if let Some((mut diff, errors)) = generate_migration_diff(files, &diags) {
        let rendered_errors = report_diagnostics_to_buffer(files, errors, /* color */ false);
        let _ = writer.write_all(&rendered_errors);
        let _ = writer.write_all(diff.render_output().as_bytes());
    } else {
        let _ = writer.write_all("No migration report".as_bytes());
    }
    writer.into_inner()
}

//**************************************************************************************************
// impls
//**************************************************************************************************

impl Diagnostics {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn max_severity(&self) -> Option<Severity> {
        let Self(Some(inner)) = self else { return None };
        // map would be empty at the severity, so it should never be zero
        debug_assert!(inner.severity_count.values().all(|count| *count > 0));
        inner
            .severity_count
            .iter()
            .max_by_key(|(sev, _count)| **sev)
            .map(|(sev, _count)| *sev)
    }

    pub fn count_diags_at_or_above_severity(&self, threshold: Severity) -> usize {
        let Self(Some(inner)) = self else { return 0 };
        // map would be empty at the severity, so it should never be zero
        debug_assert!(inner.severity_count.values().all(|count| *count > 0));
        inner
            .severity_count
            .iter()
            .filter(|(sev, _count)| **sev >= threshold)
            .map(|(_sev, count)| *count)
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        let Self(Some(inner)) = self else { return true };
        inner.diagnostics.is_empty()
    }

    pub fn len(&self) -> usize {
        let Self(Some(inner)) = self else { return 0 };
        inner.diagnostics.len()
    }

    pub fn add(&mut self, diag: Diagnostic) {
        if self.0.is_none() {
            self.0 = Some(Diagnostics_::default())
        }
        let inner = self.0.as_mut().unwrap();
        *inner
            .severity_count
            .entry(diag.info.severity())
            .or_insert(0) += 1;
        inner.diagnostics.push(diag)
    }

    pub fn add_opt(&mut self, diag_opt: Option<Diagnostic>) {
        if let Some(diag) = diag_opt {
            self.add(diag)
        }
    }

    pub fn add_source_filtered(&mut self, diag: Diagnostic) {
        if self.0.is_none() {
            self.0 = Some(Diagnostics_::default())
        }
        let inner = self.0.as_mut().unwrap();
        inner.filtered_source_diagnostics.push(diag)
    }

    pub fn extend(&mut self, other: Self) {
        let Self(Some(Diagnostics_ {
            diagnostics,
            filtered_source_diagnostics: _,
            severity_count,
        })) = other
        else {
            return;
        };
        if self.0.is_none() {
            self.0 = Some(Diagnostics_::default())
        }
        let inner = self.0.as_mut().unwrap();
        for (sev, count) in severity_count {
            *inner.severity_count.entry(sev).or_insert(0) += count;
        }
        inner.diagnostics.extend(diagnostics);
    }

    pub fn into_vec(self) -> Vec<Diagnostic> {
        self.0.map(|inner| inner.diagnostics).unwrap_or_default()
    }

    pub fn into_codespan_format(
        self,
    ) -> Vec<(
        codespan_reporting::diagnostic::Severity,
        &'static str,
        (Loc, String),
        Vec<(Loc, String)>,
        Vec<String>,
    )> {
        let mut v = vec![];
        for diag in self.into_vec() {
            let Diagnostic {
                info,
                primary_label,
                secondary_labels,
                notes,
            } = diag;
            let csr_diag = (
                info.severity().into_codespan_severity(),
                info.message(),
                primary_label,
                secondary_labels,
                notes,
            );
            v.push(csr_diag)
        }
        v
    }

    pub fn retain(&mut self, f: impl FnMut(&Diagnostic) -> bool) {
        if self.0.is_none() {
            return;
        }
        let inner = self.0.as_mut().unwrap();
        inner.diagnostics.retain(f);
    }

    pub fn any_with_prefix(&self, prefix: &str) -> bool {
        let Self(Some(inner)) = self else {
            return false;
        };
        inner
            .diagnostics
            .iter()
            .any(|d| d.info.external_prefix() == Some(prefix))
    }

    /// Returns the number of diags filtered in source (user) code (an not in
    /// the dependencies) that have a given prefix (first value returned)
    /// and how many different categories of diags were filtered.
    pub fn filtered_source_diags_with_prefix(&self, prefix: &str) -> (usize, usize) {
        let Self(Some(inner)) = self else {
            return (0, 0);
        };
        let mut filtered_diags_num = 0;
        let mut filtered_categories = HashSet::new();
        inner.filtered_source_diagnostics.iter().for_each(|d| {
            if d.info.external_prefix() == Some(prefix) {
                filtered_diags_num += 1;
                filtered_categories.insert(d.info.category());
            }
        });
        (filtered_diags_num, filtered_categories.len())
    }
}

impl Diagnostic {
    pub fn new(
        code: impl Into<DiagnosticInfo>,
        (loc, label): (Loc, impl ToString),
        secondary_labels: impl IntoIterator<Item = (Loc, impl ToString)>,
        notes: impl IntoIterator<Item = impl ToString>,
    ) -> Self {
        Diagnostic {
            info: code.into(),
            primary_label: (loc, label.to_string()),
            secondary_labels: secondary_labels
                .into_iter()
                .map(|(loc, msg)| (loc, msg.to_string()))
                .collect(),
            notes: notes.into_iter().map(|msg| msg.to_string()).collect(),
        }
    }

    pub fn set_code(mut self, code: impl Into<DiagnosticInfo>) -> Self {
        self.info = code.into();
        self
    }

    pub(crate) fn set_severity(mut self, severity: Severity) -> Self {
        self.info = self.info.set_severity(severity);
        self
    }

    #[allow(unused)]
    pub fn add_secondary_labels(
        &mut self,
        additional_labels: impl IntoIterator<Item = (Loc, impl ToString)>,
    ) {
        self.secondary_labels.extend(
            additional_labels
                .into_iter()
                .map(|(loc, msg)| (loc, msg.to_string())),
        )
    }

    pub fn add_secondary_label(&mut self, (loc, msg): (Loc, impl ToString)) {
        self.secondary_labels.push((loc, msg.to_string()))
    }

    pub fn extra_labels_len(&self) -> usize {
        self.secondary_labels.len() + self.notes.len()
    }

    #[allow(unused)]
    pub fn add_notes(&mut self, additional_notes: impl IntoIterator<Item = impl ToString>) {
        self.notes
            .extend(additional_notes.into_iter().map(|msg| msg.to_string()))
    }

    pub fn add_note(&mut self, msg: impl ToString) {
        self.notes.push(msg.to_string())
    }

    pub fn info(&self) -> &DiagnosticInfo {
        &self.info
    }

    pub fn primary_msg(&self) -> &str {
        &self.primary_label.1
    }

    pub fn is_migration(&self) -> bool {
        const MIGRATION_CATEGORY: u8 = codes::Category::Migration as u8;
        self.info.category() == MIGRATION_CATEGORY
    }
}

#[macro_export]
macro_rules! diag {
    ($code: expr, $primary: expr $(,)?) => {{
        #[allow(unused)]
        use $crate::diagnostics::codes::*;
        $crate::diagnostics::Diagnostic::new(
            $code,
            $primary,
            std::iter::empty::<(move_ir_types::location::Loc, String)>(),
            std::iter::empty::<String>(),
        )
    }};
    ($code: expr, $primary: expr, $($secondary: expr),+ $(,)?) => {{
        #[allow(unused)]
        use $crate::diagnostics::codes::*;
        $crate::diagnostics::Diagnostic::new(
            $code,
            $primary,
            vec![$($secondary, )*],
            std::iter::empty::<String>(),
        )
    }};
}

pub const ICE_BUG_REPORT_MESSAGE: &str = "The Move compiler has encountered an internal compiler error.\n \
    Please report this this issue to the IOTA Foundation Move language team,\n \
    including this error and any relevant code, to the IOTA Foundation issue tracker\n \
    at : https://github.com/iotaledger/iota/issues";

#[macro_export]
macro_rules! ice {
    ($primary: expr $(,)?) => {{
        $crate::diagnostics::print_stack_trace();
        let mut diag = $crate::diag!($crate::diagnostics::codes::Bug::ICE, $primary);
        diag.add_note($crate::diagnostics::ICE_BUG_REPORT_MESSAGE.to_string());
        diag
    }};
    ($primary: expr, $($secondary: expr),+ $(,)?) => {{
        $crate::diagnostics::print_stack_trace();
        let mut diag =
            $crate::diag!($crate::diagnostics::codes::Bug::ICE, $primary, $($secondary, )*);
        diag.add_note($crate::diagnostics::ICE_BUG_REPORT_MESSAGE.to_string());
        diag
    }}
}

#[macro_export]
macro_rules! ice_assert {
    ($env: expr, $cond: expr, $loc: expr, $($arg:tt)*) => {{
        if !$cond {
            $env.add_diag($crate::ice!((
                $loc,
                format!($($arg)*),
            )));
        }
    }}
}

#[allow(clippy::wildcard_in_or_patterns)]
pub fn print_stack_trace() {
    use std::backtrace::{Backtrace, BacktraceStatus};
    let stacktrace = Backtrace::capture();
    match stacktrace.status() {
        BacktraceStatus::Captured => {
            eprintln!("stacktrace:");
            eprintln!("{}", stacktrace);
        }
        BacktraceStatus::Unsupported | BacktraceStatus::Disabled | _ => (),
    }
}

impl WarningFilters {
    pub fn new_for_source() -> Self {
        Self {
            filters: BTreeMap::new(),
            for_dependency: false,
        }
    }

    pub fn new_for_dependency() -> Self {
        Self {
            filters: BTreeMap::new(),
            for_dependency: true,
        }
    }

    pub fn is_filtered(&self, diag: &Diagnostic) -> bool {
        self.is_filtered_by_info(&diag.info)
    }

    fn is_filtered_by_info(&self, info: &DiagnosticInfo) -> bool {
        let prefix = info.external_prefix();
        self.filters
            .get(&prefix)
            .is_some_and(|filters| filters.is_filtered_by_info(info))
    }

    pub fn union(&mut self, other: &Self) {
        for (prefix, filters) in &other.filters {
            self.filters
                .entry(*prefix)
                .or_insert_with(UnprefixedWarningFilters::new)
                .union(filters);
        }
        // if there is a dependency code filter on the stack, it means we are filtering
        // dependent code and this information must be preserved when stacking
        // up additional filters (which involves union of the current filter
        // with the new one)
        self.for_dependency = self.for_dependency || other.for_dependency;
    }

    pub fn add(&mut self, filter: WarningFilter) {
        let (prefix, category, code, name) = match filter {
            WarningFilter::All(prefix) => {
                self.filters.insert(prefix, UnprefixedWarningFilters::All);
                return;
            }
            WarningFilter::Category {
                prefix,
                category,
                name,
            } => (prefix, category, None, name),
            WarningFilter::Code {
                prefix,
                category,
                code,
                name,
            } => (prefix, category, Some(code), name),
        };
        self.filters
            .entry(prefix)
            .or_insert(UnprefixedWarningFilters::Empty)
            .add(category, code, name)
    }

    pub fn unused_warnings_filter_for_test() -> Self {
        Self {
            filters: BTreeMap::from([(
                None,
                UnprefixedWarningFilters::unused_warnings_filter_for_test(),
            )]),
            for_dependency: false,
        }
    }

    pub fn for_dependency(&self) -> bool {
        self.for_dependency
    }
}

impl UnprefixedWarningFilters {
    pub fn new() -> Self {
        Self::Empty
    }

    fn is_filtered_by_info(&self, info: &DiagnosticInfo) -> bool {
        match self {
            Self::All => info.severity() == Severity::Warning,
            Self::Specified { categories, codes } => {
                info.severity() == Severity::Warning
                    && (categories.contains_key(&info.category())
                        || codes.contains_key(&(info.category(), info.code())))
            }
            Self::Empty => false,
        }
    }

    pub fn union(&mut self, other: &Self) {
        match (self, other) {
            // if self is empty, just take the other filter
            (s @ Self::Empty, _) => *s = other.clone(),
            // if other is empty, or self is ALL, no change to the filter
            (_, Self::Empty) => (),
            (Self::All, _) => (),
            // if other is all, self is now all
            (s, Self::All) => *s = Self::All,
            // category and code level union
            (
                Self::Specified { categories, codes },
                Self::Specified {
                    categories: other_categories,
                    codes: other_codes,
                },
            ) => {
                categories.extend(other_categories);
                // remove any codes covered by the category level filter
                codes.extend(
                    other_codes
                        .iter()
                        .filter(|((category, _), _)| !categories.contains_key(category)),
                );
            }
        }
    }

    /// Add a specific filter to the filter map.
    /// If filter_code is None, then the filter applies to all codes in the
    /// filter_category.
    fn add(
        &mut self,
        filter_category: u8,
        filter_code: Option<u8>,
        filter_name: Option<WellKnownFilterName>,
    ) {
        match self {
            Self::All => (),
            Self::Empty => {
                *self = Self::Specified {
                    categories: BTreeMap::new(),
                    codes: BTreeMap::new(),
                };
                self.add(filter_category, filter_code, filter_name)
            }
            Self::Specified { categories, .. } if categories.contains_key(&filter_category) => (),
            Self::Specified { categories, codes } => {
                if let Some(filter_code) = filter_code {
                    codes.insert((filter_category, filter_code), filter_name);
                } else {
                    categories.insert(filter_category, filter_name);
                    codes.retain(|(category, _), _| *category != filter_category);
                }
            }
        }
    }

    pub fn unused_warnings_filter_for_test() -> Self {
        let filtered_codes = [
            (UnusedItem::Function, FILTER_UNUSED_FUNCTION),
            (UnusedItem::StructField, FILTER_UNUSED_STRUCT_FIELD),
            (UnusedItem::FunTypeParam, FILTER_UNUSED_TYPE_PARAMETER),
            (UnusedItem::Constant, FILTER_UNUSED_CONST),
            (UnusedItem::MutReference, FILTER_UNUSED_MUT_REF),
            (UnusedItem::MutParam, FILTER_UNUSED_MUT_PARAM),
        ]
        .into_iter()
        .map(|(item, filter)| {
            let info = item.into_info();
            ((info.category(), info.code()), Some(filter))
        })
        .collect();
        Self::Specified {
            categories: BTreeMap::new(),
            codes: filtered_codes,
        }
    }
}

impl Migration {
    pub fn new(
        sources: FilesSourceText,
        diags: Vec<Diagnostic>,
    ) -> (Migration, /* Migration errors */ Diagnostics) {
        let mapped_files = MappedFiles::new(sources);
        let mut mig = Migration {
            changes: BTreeMap::new(),
            mapped_files,
        };

        let migration_errors = Diagnostics::new();
        for diag in diags {
            mig.add_diagnostic(diag);
        }

        (mig, migration_errors)
    }

    fn add_diagnostic(&mut self, diag: Diagnostic) {
        const CAT: u8 = Category::Migration as u8;
        const NEEDS_MUT: u8 = codes::Migration::NeedsLetMut as u8;
        const NEEDS_PUBLIC: u8 = codes::Migration::NeedsPublic as u8;
        const NEEDS_BACKTICKS: u8 = codes::Migration::NeedsRestrictedIdentifier as u8;
        const NEEDS_GLOBAL_QUAL: u8 = codes::Migration::NeedsGlobalQualification as u8;
        const REMOVE_FRIEND: u8 = codes::Migration::RemoveFriend as u8;
        const MAKE_PUB_PACKAGE: u8 = codes::Migration::MakePubPackage as u8;
        const ADDRESS_REMOVE: u8 = codes::Migration::AddressRemove as u8;
        const ADDRESS_ADD: u8 = codes::Migration::AddressAdd as u8;

        let FileByteSpan { file_id, byte_span } = self.find_file_location(&diag);
        let file_change_entry = self.changes.entry(file_id).or_default();
        let change = match (diag.info().category(), diag.info().code()) {
            (CAT, NEEDS_MUT) => MigrationChange::AddMut,
            (CAT, NEEDS_PUBLIC) => MigrationChange::AddPublic,
            (CAT, NEEDS_BACKTICKS) => {
                let old_name = diag.primary_msg().to_string();
                MigrationChange::Backquote(old_name)
            }
            (CAT, NEEDS_GLOBAL_QUAL) => MigrationChange::AddGlobalQual,
            (CAT, REMOVE_FRIEND) => MigrationChange::RemoveFriend,
            (CAT, MAKE_PUB_PACKAGE) => MigrationChange::MakePubPackage,
            (CAT, ADDRESS_REMOVE) => MigrationChange::AddressRemove,
            (CAT, ADDRESS_ADD) => {
                let insertion = diag.primary_msg().to_string();
                MigrationChange::AddressAdd(insertion)
            }
            _ => unreachable!(),
        };
        file_change_entry.push((byte_span, change));
    }

    fn find_file_location(&mut self, diag: &Diagnostic) -> FileByteSpan {
        let (loc, _msg) = &diag.primary_label;
        self.mapped_files.byte_location(*loc)
    }

    fn get_file_contents(&self, file_id: FileId) -> String {
        self.mapped_files.files.source(file_id).unwrap().to_string()
    }

    fn render_changes(source: String, changes: &mut [(ByteSpan, MigrationChange)]) -> String {
        changes.sort_by(|(loc0, _), (loc1, _)| loc0.start.partial_cmp(&loc1.start).unwrap());
        let mut output = "".to_string();

        let mut source_prefix = &source[..];
        let mut last_seen = source_prefix.len();
        for (loc, change) in changes.iter().rev() {
            assert!(loc.end <= last_seen, "Found overlapping migrations.");
            match change {
                MigrationChange::AddMut => {
                    let rest = &source_prefix[loc.start..];
                    output = format!("mut {}{}", rest, output);
                }
                MigrationChange::AddPublic => {
                    let rest = &source_prefix[loc.start..];
                    output = format!("public {}{}", rest, output);
                }
                MigrationChange::Backquote(old_name) => {
                    let rest = &source_prefix[loc.end..];
                    output = format!("`{}`{}{}", old_name, rest, output);
                }
                MigrationChange::AddGlobalQual => {
                    let rest = &source_prefix[loc.start..];
                    output = format!("::{}{}", rest, output);
                }
                MigrationChange::RemoveFriend => {
                    let rest = &source_prefix[loc.end..];
                    output = format!(
                        "/* {} */{}{}",
                        &source_prefix[loc.start..loc.end],
                        rest,
                        output
                    );
                }
                MigrationChange::MakePubPackage => {
                    let rest = &source_prefix[loc.end..];
                    output = format!("public(package){}{}", rest, output);
                }
                MigrationChange::AddressRemove => {
                    let rest = &source_prefix[loc.end..];
                    output = format!(
                        "/* {} */{}{}",
                        &source_prefix[loc.start..loc.end],
                        rest,
                        output
                    );
                }
                MigrationChange::AddressAdd(insertion) => {
                    let rest = &source_prefix[loc.start..];
                    output = format!("{}{}{}", insertion, rest, output);
                }
            }
            source_prefix = &source_prefix[..loc.start];
            last_seen = loc.start;
        }
        output = format!("{}{}", source_prefix, output);

        output
    }

    pub fn render_output(&mut self) -> String {
        let mut output = vec![];
        let mut names = self
            .changes
            .keys()
            .map(|id| (*id, *self.mapped_files.files.get(*id).unwrap().name()))
            .collect::<Vec<_>>();
        names.sort_by_key(|(_, name)| *name);
        for (file_id, name) in names {
            let original = self.get_file_contents(file_id);
            let file_changes = self.changes.get_mut(&file_id).unwrap();
            Self::ensure_unique_changes(file_changes);
            let migrated = Self::render_changes(original.clone(), file_changes);
            let diff = similar::TextDiff::from_lines(&original, &migrated);
            output.push(
                diff.unified_diff()
                    .context_radius(0)
                    .header(&name, &name)
                    .to_string(),
            );
        }

        output.join("")
    }

    pub fn record_diff(&mut self, path: PathBuf) -> anyhow::Result<String> {
        let output_path = path.join("migration.patch");
        let string_result = output_path.to_str().unwrap_or("invalid path").to_string();
        std::fs::write(output_path, self.render_output())?;
        Ok(string_result)
    }

    pub fn apply_changes<W: Write>(&mut self, w: &mut W) -> anyhow::Result<()> {
        writeln!(w)?;
        let mut names = self
            .changes
            .keys()
            .map(|id| (*id, *self.mapped_files.files.get(*id).unwrap().name()))
            .collect::<Vec<_>>();
        names.sort_by_key(|(_, name)| *name);
        for (file_id, name) in names {
            let original = self.get_file_contents(file_id);
            let file_changes = self.changes.get_mut(&file_id).unwrap();
            Self::ensure_unique_changes(file_changes);
            let migrated = Self::render_changes(original.clone(), file_changes);
            let path = PathBuf::from(name.to_string());
            writeln!(w, "Updating {:#?} . . .", path)?;
            Self::ensure_unique_changes(file_changes);
            std::fs::write(path, migrated)?;
        }
        Ok(())
    }

    fn ensure_unique_changes(changes: &mut Vec<(ByteSpan, MigrationChange)>) {
        let file_changes = std::mem::take(changes);
        let mut set_changes = BTreeSet::new();
        for change in file_changes {
            set_changes.insert(change);
        }
        let out_changes = set_changes.into_iter().collect::<Vec<_>>();
        let _ = std::mem::replace(changes, out_changes);
    }
}

//**************************************************************************************************
// traits
//**************************************************************************************************

impl FromIterator<Diagnostic> for Diagnostics {
    fn from_iter<I: IntoIterator<Item = Diagnostic>>(iter: I) -> Self {
        let diagnostics = iter.into_iter().collect::<Vec<_>>();
        Self::from(diagnostics)
    }
}

impl From<Vec<Diagnostic>> for Diagnostics {
    fn from(diagnostics: Vec<Diagnostic>) -> Self {
        if diagnostics.is_empty() {
            return Self(None);
        }

        let mut severity_count = BTreeMap::new();
        for diag in &diagnostics {
            *severity_count.entry(diag.info.severity()).or_insert(0) += 1;
        }
        Self(Some(Diagnostics_ {
            diagnostics,
            filtered_source_diagnostics: vec![],
            severity_count,
        }))
    }
}

impl From<Option<Diagnostic>> for Diagnostics {
    fn from(diagnostic_opt: Option<Diagnostic>) -> Self {
        Diagnostics::from(diagnostic_opt.map_or_else(Vec::new, |diag| vec![diag]))
    }
}

impl AstDebug for WarningFilters {
    fn ast_debug(&self, w: &mut crate::shared::ast_debug::AstWriter) {
        for (prefix, filters) in &self.filters {
            let prefix_str = prefix.unwrap_or(known_attributes::DiagnosticAttribute::ALLOW);
            match filters {
                UnprefixedWarningFilters::All => w.write(&format!(
                    "#[{}({})]",
                    prefix_str,
                    WarningFilter::All(*prefix).to_str().unwrap(),
                )),
                UnprefixedWarningFilters::Specified { categories, codes } => {
                    w.write(&format!("#[{}(", prefix_str));
                    let items = categories
                        .iter()
                        .map(|(cat, n)| WarningFilter::Category {
                            prefix: *prefix,
                            category: *cat,
                            name: *n,
                        })
                        .chain(codes.iter().map(|((cat, code), n)| WarningFilter::Code {
                            prefix: *prefix,
                            category: *cat,
                            code: *code,
                            name: *n,
                        }));
                    w.list(items, ",", |w, filter| {
                        w.write(filter.to_str().unwrap());
                        false
                    });
                    w.write(")]")
                }
                UnprefixedWarningFilters::Empty => (),
            }
        }
    }
}

impl<C: DiagnosticCode> From<C> for DiagnosticInfo {
    fn from(value: C) -> Self {
        value.into_info()
    }
}
