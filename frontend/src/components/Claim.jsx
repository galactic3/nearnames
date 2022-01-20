import React, {useRef, useState} from 'react';
import { generateSeedPhrase, parseSeedPhrase } from 'near-seed-phrase';
import Loader from "./Loader";
import {BOATLOAD_OF_GAS, renderName} from "../utils";
import {Modal, Box, IconButton} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";

function ModalClaim(props) {

  const [seedPhrase, setSeedPhrase] = useState(generateSeedPhrase().seedPhrase);
  const [showSuccess, setShowSuccess] = useState(false);
  const [showLoader, setShowLoader] = useState(false);
  const [claimedBySeedPhrase, setClaimedBySeedPhrase] = useState(false);
  const [publicKey, setPublicKey] = useState('');

  const seedPhraseRef = useRef(null);
  const publicKeyRef = useRef(null);

  const lot_id = props.lot.lot_id;

  const recoverLink = props.config.walletUrl + '/recover-seed-phrase'

  const claimLot = async (publicKey) => {
    await props.contract.lot_claim({'lot_id': lot_id, 'public_key': publicKey}, BOATLOAD_OF_GAS).then((lot) => {
      setShowLoader(false);
    });
  }

  const claimBySeedPhrase = async (e) => {
    e.preventDefault()
    const seedPhrase = seedPhraseRef.current.value;
    setSeedPhrase(seedPhrase);
    const publicKey = parseSeedPhrase(seedPhrase, '').publicKey;
    setShowLoader(true);
    await claimLot(publicKey);
    setClaimedBySeedPhrase(true);
    setPublicKey(publicKey);
    setShowSuccess(true);
  }

  const claimByPublicKey = async (e) => {
    e.preventDefault()
    const publicKey = publicKeyRef.current.value;
    console.log(publicKey);
    await claimLot(publicKey);
    setPublicKey(publicKey);
    setShowSuccess(true)
  }

  const clearState = () => {
    if (showLoader) {
      return;
    }
    setPublicKey('');
    setSeedPhrase(generateSeedPhrase().seedPhrase);
    setShowSuccess(false);
    props.onClose();
  }

  return (
    <Modal {...props} onClose={() => clearState()}>
      <Box className="modal-container claim_modal">
        <IconButton
          aria-label="close"
          onClick={() => clearState()}
          className="button-icon"
          sx={{
            position: 'absolute',
            right: 8,
            top: 8,
            color: 'var(--gray)',
          }}
        >
          <CloseIcon />
        </IconButton>
        <div>
          <h3>Claim <strong>{renderName(lot_id)}</strong></h3>
          { showLoader && <Loader/> }
          { !showSuccess && !showLoader ? <div>
              <form onSubmit={(e) => claimBySeedPhrase(e)}>
                <div className='form-group'>
                  <label>Save this randomly generated seed phrase or choose your own</label>
                  <textarea
                    className='form-control'
                    defaultValue={seedPhrase}
                    ref={seedPhraseRef}
                  />
                  <button className='full-width' type="submit">Claim using seed phrase</button>
                </div>
              </form>
              <span className="or"></span>
              <form onSubmit={(e) => claimByPublicKey(e)}>
                <div className='form-group'>
                  <label>Put your base58 public key</label>
                  <textarea
                    className='form-control'
                    ref={publicKeyRef}
                    required
                  />
                  <button className='full-width' type="submit">Claim using new public key</button>
                </div>
              </form>
            </div> :
            <div className="claim-success">
              {claimedBySeedPhrase && <div><label>Seed phrase used:</label><span className='textarea green'>{seedPhrase}</span></div>}
              {claimedBySeedPhrase && <p>Go to <a target="_blank" href={recoverLink}>wallet</a> and restore your account</p>}
              {showSuccess && <div><label>Successfully Added public key </label><span className='textarea red'>{publicKey}</span></div>}
            </div>
          }
        </div>
      </Box>
    </Modal>
  );
}

export default ModalClaim;