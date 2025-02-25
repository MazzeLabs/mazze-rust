

use super::storage::Error as StorageError;
use mazze_types::Address;
use primitives::account::AccountError;
use rlp::DecoderError;

error_chain! {
    links {
    }

    foreign_links {
        Account(AccountError);
        Storage(StorageError);
        Decoder(DecoderError);
    }

    errors {
        IncompleteDatabase(address: Address) {
            description("incomplete database")
            display("incomplete database: address={:?}", address)
        }

        PosDatabaseError(err: String) {
            description("PoS database error")
            display("PoS database error, err={:?}", err)
        }
    }
}
