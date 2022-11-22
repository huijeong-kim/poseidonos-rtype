use strum_macros::Display;

#[derive(Display, Debug, PartialEq)]
pub enum PosEventId {
    SUCCESS,
    ERROR,

    POS_TRACE_STARTED,
    POS_TRACE_INIT_FAIL,
    UPDATE_ABR_DEBUG_MSG,
    POS_TRACE_ARRAY_CREATED,
    UBIO_WITHOUT_UBLOCKDEV,

    MBR_DEBUG_MSG,
    MBR_ABR_NOT_FOUND,
    MBR_DEVICE_ALREADY_IN_ARRAY,
    MBR_DEVICE_NOT_FOUND,
    MBR_DATA_NOT_FOUND,
    MBR_READ_DONE,
    MBR_WRONG_ARRAY_INDEX_MAP,
    MBR_MAX_ARRAY_CNT_EXCEED,
    MBR_ABR_ALREADY_EXIST,
    MBR_WRONG_ARRAY_VALID_FLAG,
    POS_TRACE_MBR_LOADED,

    CREATE_ARRAY_SAME_ARRAY_NAME_EXISTS,
    CREATE_ARRAY_EXCEED_MAX_NUM_OF_ARRAYS,
    CREATE_ARRAY_NAME_TOO_SHORT,
    CREATE_ARRAY_NAME_TOO_LONG,
    CREATE_ARRAY_NAME_INCLUDES_SPECIAL_CHAR,
    CREATE_ARRAY_NAME_START_OR_END_WITH_SPACE,

    CREATE_ARRAY_NOT_SUPPORTED_RAIDTYPE,
    CREATE_ARRAY_RAID_DOES_NOT_SUPPORT_SPARE_DEV,

    UPDATE_ARRAY_NAME_TOO_LONG,
    UPDATE_CREATION_DATE_TOO_LONG,
    UPDATE_MODIFIED_DATE_TOO_LONG,

    DELETE_ARRAY_ARRAY_NAME_DOES_NOT_EXIST,
    DEVICEMGR_DEVICE_NOT_FOUND,

    CREATE_ARRAY_DEBUG_MSG,

    MBR_WRITE_ERROR,

    ARRAY_EVENT_DEV_STATE_CHANGED,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_to_string() {
        assert_eq!("SUCCESS".to_string(), PosEventId::SUCCESS.to_string());
        assert_eq!(
            "POS_TRACE_STARTED".to_string(),
            PosEventId::POS_TRACE_STARTED.to_string()
        );
    }
}
