use std::any::Any;
use std::borrow::BorrowMut;
use std::rc::Rc;
use std::sync::Mutex;

use crossbeam::sync::Parker;
use log::{info, warn};

use crate::array::meta::array_meta::ArrayMeta;
use crate::bio::ubio::{CallbackClosure, Ubio, UbioDir};
use crate::device::base::ublock_device::UBlockDevice;
use crate::event_scheduler::callback::Callback;
use crate::include::meta_const::CHUNK_SIZE;
use crate::include::pos_event_id::PosEventId;
use crate::io_scheduler::io_dispatcher::IODispatcherSingleton;
use crate::mbr::mbr_info::{ArrayBootRecord, IntoVecOfU8, masterBootRecord};

const MBR_CHUNKS : i32 = 1;
const MBR_ADDRESS: u64 = 0;
const MBR_SIZE: u64 = CHUNK_SIZE;

pub struct MbrManager {
    mbrBuffer: Mutex<Vec<u8>>,
    systeminfo: masterBootRecord,
}

impl MbrManager {
    pub fn new() -> Self {
        MbrManager {
            mbrBuffer: Mutex::new(vec![0 as u8; CHUNK_SIZE as usize]),
            systeminfo: Default::default(),
        }
    }

    pub fn GetMbr(&self) -> masterBootRecord { todo!(); }
    pub fn LoadMbr(&self) -> Result<(), PosEventId> { todo!();  }
    pub fn SaveMbr(&self) -> Result<(), PosEventId> { todo!();  }
    pub fn ResetMbr(&self) -> Result<(), PosEventId> { todo!(); }

    pub fn InitDisk(&self, dev: Box<dyn UBlockDevice>) {
        let mut mbrBuffer = self.mbrBuffer.lock().unwrap(); //.borrow_mut();
        mbrBuffer.clear();
        let mut systeminfo = self.systeminfo.to_vec_u8();
        mbrBuffer.append(&mut systeminfo);
        self._SetParity(&mbrBuffer);
        let diskIoCtxt = DiskIoContext::new(UbioDir::Write, mbrBuffer.clone());
        self._DiskIo(dev, diskIoCtxt);
        info!("the mbr has been initialized");
    }

    pub fn CreateAbr(&self, meta: ArrayMeta) -> Result<(), PosEventId> { todo!(); }
    pub fn DeleteAbr(&self, name: &String) -> Result<(), PosEventId> { todo!(); }
    pub fn GetAbr(&self, name: &String) -> Option<(ArrayBootRecord, u32)> { todo!(); }
    pub fn GetAbrList(&self) -> Result<Vec::<ArrayBootRecord>, PosEventId> { todo!(); }
    pub fn GetMbrVersionInMemory(&self) -> Result<i32, PosEventId> { todo!(); }
    pub fn UpdateDeviceIndexMap(&self, arrayName: &String) -> Result<(), PosEventId> { todo!(); }
    pub fn FindArrayWithDeviceSN(&self, devSN: String) -> String { String::new() }
    pub fn Serialize(&self) -> String { todo!(); }

    fn _IterateReadFromDevices(&self, dev: Box<dyn UBlockDevice>, ctx: &mut Vec<Vec<u8>>/*Box<dyn Any>*/) {
        // "ctx" is likely to be byte buffer, so can be refactored accordingly later.
        let mut mems = ctx;
        let mem = [0 as u8; CHUNK_SIZE as usize * MBR_CHUNKS as usize].to_vec();
        let diskIoCtxt = DiskIoContext::new(UbioDir::Read, mem);
        let result_buffer = self._DiskIo(dev, diskIoCtxt)
            .expect("Failed to read MBR from a device"); // TODO: device id API 생기면 메시지에 추가
        if !self._VerifyParity(&result_buffer) {
            warn!("Failed to verify MBR parity");
            return;
        }
        if !self._VerifySystemUuid(&result_buffer) {
            warn!("Failed to verify System UUID from MBR");
            return;
        }
        mems.push(result_buffer);
    }

    fn _DiskIo(&self, dev: Box<dyn UBlockDevice>, ctx: DiskIoContext) -> Option<Vec<u8>> {
        let result_buffer = Rc::new(Mutex::new(Vec::new()));
        let io_done_parker = Parker::new();
        let io_done_unparker = io_done_parker.unparker().clone();
        let io_dir = ctx.ubioDir;
        let callback: CallbackClosure = match io_dir {
            UbioDir::Read => {
                let mut result_buffer = result_buffer.clone();
                Box::new(
                    move |read_buffer: &Vec<u8>| {
                        let mut result_buffer = result_buffer.lock().unwrap();
                        for the_byte in read_buffer {
                            result_buffer.push(the_byte.clone());
                        }
                        io_done_unparker.unpark();
                    }
                )
            },
            UbioDir::Write => {
                Box::new(
                    move |_: &Vec<u8>| {
                        io_done_unparker.unpark();
                    }
                )
            },
        };

        let mut bio = Ubio::new(io_dir.clone(), MBR_ADDRESS,
                                ctx.mem, callback);
        bio.uBlock = Some(dev);

        IODispatcherSingleton.lock().unwrap().Submit(bio, true /* not used */, false);

        io_done_parker.park(); // block synchronously here until we get "unparked"
        match io_dir {
            UbioDir::Read => {
                Some(result_buffer.lock().unwrap().clone())
            }
            UbioDir::Write => {
                None
            }
        }
    }

    fn _VerifyParity(&self, mem: &Vec<u8>) -> bool {
        // TODO
        true
    }

    fn _VerifySystemUuid(&self, mem: &Vec<u8>) -> bool {
        // TODO
        true
    }

    fn _SetParity(&self, mem: &Vec<u8>) {
        // TODO
    }
}

struct DiskIoContext {
    ubioDir: UbioDir,
    mem: Vec<u8>,
}

impl DiskIoContext {
    pub fn new(ubioDir: UbioDir, mem: Vec<u8>) -> DiskIoContext {
        DiskIoContext {
            ubioDir,
            mem
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use std::path::PathBuf;

    use crate::bio::ubio::{CallbackClosure, Ubio, UbioDir};
    use crate::device::base::ublock_device::UBlockDevice;
    use crate::device::device_manager::DeviceManager;
    use crate::device::ufile::ufile_ssd::UfileSsd;
    use crate::mbr::mbr_info::MBR_VERSION_OFFSET;
    use crate::mbr::mbr_manager::{DiskIoContext, MBR_ADDRESS, MbrManager};

    fn setup() {
        // set up the logger for the test context
        env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_if_reading_from_MBR_of_a_single_device_works() {
        setup();

        let test_ufile_ssd = "/tmp/test-ufile-ssd";
        fs::remove_file(PathBuf::from(test_ufile_ssd));

        // Given: a UBlockDevice with its MBR filled with a specific pattern
        let PATTERN = vec![0, 2, 4, 6, 1, 3, 5, 7];
        let empty_callback = Box::new(move |_: &Vec<u8>| {});
        let mut ubio = Ubio::new(UbioDir::Write, MBR_ADDRESS, PATTERN.clone(), empty_callback);
        let mut ublock_dev = UfileSsd::new(
            PathBuf::from(test_ufile_ssd), 100*1024*1024)
            .boxed();
        ublock_dev.Open();
        ublock_dev.SubmitAsyncIO(&mut ubio);
        let mbr_manager = MbrManager::new();
        let mut ctx : Vec<Vec<u8>> = Vec::new();

        // When: MBR manager reads MBR from the device
        mbr_manager._IterateReadFromDevices(ublock_dev, &mut ctx);

        // Then: "ctx" should contain the pattern
        assert_eq!(1, ctx.len());
        let mbr = &ctx[0];
        let expected = PATTERN.to_vec();
        let actual = mbr[0..PATTERN.len()].to_vec();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_if_InitDisk_actually_writes_MBR_to_a_given_device() {
        setup();
        let test_ufile_ssd = "/tmp/test-ufile-ssd-initdisk";
        fs::remove_file(PathBuf::from(test_ufile_ssd));

        // Given: MBR manager with its MBR version filled with 123
        let mut mbr_manager = MbrManager::new();
        let mbr = &mut mbr_manager.systeminfo;
        mbr.mbrVersion = 123;

        let mut ublock_dev = UfileSsd::new(
            PathBuf::from(test_ufile_ssd), 100*1024*1024)
            .boxed();
        ublock_dev.Open();

        // When: MBR manager initializes a given UBlockDevice
        mbr_manager.InitDisk(ublock_dev);

        // Then: We should be able to see the expected MBR version 123 at byte offset 32 within the UfileSsd
        let mut f = File::open(PathBuf::from(test_ufile_ssd)).unwrap();
        f.seek(SeekFrom::Start(MBR_VERSION_OFFSET as u64));
        let mut buf = [0 as u8; 4];
        let bytes_read = f.read(&mut buf).unwrap();
        assert_eq!(4, bytes_read);
        assert_eq!(buf.to_vec(), 123_u32.to_le_bytes().to_vec());
    }
}