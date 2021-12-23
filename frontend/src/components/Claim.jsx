import React, {useRef, useState} from 'react';
import { Modal } from "react-bootstrap";
import { generateSeedPhrase, parseSeedPhrase } from 'near-seed-phrase';
import Loader from "./Loader";
import { BOATLOAD_OF_GAS } from "../utils";

function ModalClaim(props) {

  const [seedPhrase, setSeedPhrase] = useState(generateSeedPhrase().seedPhrase);
  const [showSuccess, setShowSuccess] = useState(false);
  const [showLoader, setShowLoader] = useState(false);
  const [claimedBySeedPhrase, setClaimedBySeedPhrase] = useState(false);
  const [publicKey, setPublicKey] = useState('');

  const seedPhraseRef = useRef(null);
  const publicKeyRef = useRef(null);

  const recoverLink = props.config.walletUrl + '/recover-seed-phrase'

  const claimLot = async (publicKey) => {
    await props.contract.lot_claim({'lot_id': props.lot, 'public_key': publicKey}, BOATLOAD_OF_GAS).then((lot) => {
      console.log(lot);
      setShowLoader(false);
    });
  }

  const claimBySeedPhrase = async (e) => {
    e.preventDefault()
    const seedPhrase = seedPhraseRef.current.value;
    setSeedPhrase(seedPhrase);
    const publicKey = parseSeedPhrase(seedPhrase, '').publicKey;
    console.log(publicKey);
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

  return (
    <Modal
      {...props}
      size='lg'
      aria-labelledby="contained-modal-title-vcenter"
      centered
    >
      <Modal.Body>
        <div className='container'>
          <h3 className="my-3">Claim <strong>{props.lot.lot_id}</strong></h3>
          { showLoader && <Loader/> }
          { !showSuccess && !showLoader ? <div>
            <form onSubmit={(e) => claimBySeedPhrase(e)}>
              <div className='form-group'>
                <label>Save this randomly generated seed phrase or choose your own</label>
                <textarea
                  className='form-control my-2'
                  defaultValue={seedPhrase}
                  ref={seedPhraseRef}
                />
                <button className='w-100' type="submit">Claim using seed phrase</button>
              </div>
            </form>

            <form onSubmit={(e) => claimByPublicKey(e)}>
              <h3 className='text-center'>or</h3>
              <div className='form-group'>
                <label>Put your base58 public key</label>
                <textarea
                  className='form-control my-2'
                  ref={publicKeyRef}
                  required
                />
                <button className='w-100' type="submit">Claim using new public key</button>
              </div>
            </form>
        </div> :
        <div>
          {claimedBySeedPhrase && <div><p>Seed phrase used:</p><h5 className='alert alert-info' role='alert'>{seedPhrase}</h5></div>}
          {claimedBySeedPhrase && <p>Go to <a target="_blank" href={recoverLink}>wallet</a> and restore your account</p>}
          {showSuccess && <p>Successfully Added public key <span className='link-danger text-break'>{publicKey}</span></p>}
        </div>
        }
        </div>
      </Modal.Body>
    </Modal>
  );
}

export default ModalClaim;