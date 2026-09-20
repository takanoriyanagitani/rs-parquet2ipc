use std::fs::File;
use std::io;

use io::BufWriter;
use io::Write;

use arrow_schema::Schema;
use arrow_schema::SchemaRef;

use arrow_array::RecordBatch;
use arrow_array::RecordBatchReader;

use arrow_ipc::writer::StreamWriter;

use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

pub struct IpcWriter<W>(pub StreamWriter<W>);

impl<W> IpcWriter<W>
where
    W: Write,
{
    pub fn write_all<I>(mut self, irbat: I) -> Result<(), io::Error>
    where
        I: Iterator<Item = Result<RecordBatch, io::Error>>,
    {
        for rslt in irbat {
            let rbat: RecordBatch = rslt?;
            self.0.write(&rbat).map_err(io::Error::other)?;
        }
        self.0.flush().map_err(io::Error::other)?;
        self.0.finish().map_err(io::Error::other)?;
        let mut w: W = self.0.into_inner().map_err(io::Error::other)?;
        w.flush()
    }
}

pub struct PqReader(pub ParquetRecordBatchReader);

impl PqReader {
    pub fn schema_ref(&self) -> SchemaRef {
        self.0.schema()
    }
}

impl PqReader {
    pub fn into_writer<W>(self, mut wtr: W) -> Result<(), io::Error>
    where
        W: Write,
    {
        let sref: SchemaRef = self.schema_ref();
        let sch: &Schema = &sref;
        let swtr =
            StreamWriter::try_new(BufWriter::new(&mut wtr), sch).map_err(io::Error::other)?;
        let irbat = self.0.map(|rslt| rslt.map_err(io::Error::other));
        IpcWriter(swtr).write_all(irbat)?;
        wtr.flush()
    }

    pub fn into_stdout(self) -> Result<(), io::Error> {
        self.into_writer(io::stdout().lock())
    }
}

pub const BATCH_SIZE_DEFAULT: usize = 8192;
pub const OFFSET_DEFAULT: usize = 0;

pub struct PqConfig {
    pub batch_size: usize,
    pub limit: Option<usize>,
    pub offset: usize,

    pub pqfilename: String,
}

impl PqConfig {
    pub fn to_reader(&self) -> Result<ParquetRecordBatchReader, io::Error> {
        let f: File = File::open(&self.pqfilename)?;
        let mut bldr: ParquetRecordBatchReaderBuilder<_> =
            ParquetRecordBatchReaderBuilder::try_new(f).map_err(io::Error::other)?;
        if let Some(lmt) = self.limit {
            bldr = bldr.with_limit(lmt);
        }

        bldr.with_batch_size(self.batch_size)
            .with_offset(self.offset)
            .build()
            .map_err(io::Error::other)
    }
}

impl PqConfig {
    pub fn reader2stdout(&self) -> Result<(), io::Error> {
        let rdr: ParquetRecordBatchReader = self.to_reader()?;
        PqReader(rdr).into_stdout()
    }
}
