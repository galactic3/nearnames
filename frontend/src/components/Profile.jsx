import React, { useEffect, useState } from 'react';
import { useParams } from 'react-router';
import Lot from "./Lot";
import Loader from './Loader';
import {BOATLOAD_OF_GAS, nearTo} from "../utils";

function Profile (props) {
  const { profileId } = useParams();
  const [profile, setProfile] = useState([]);
  const [lotsOffering, setLotsOffering] = useState([]);
  const [lotsBidding, setLotsBidding] = useState([]);
  const [loader, setLoader] = useState(false);
  const [claimLoader, setClaimLoader] = useState(false);

  const contract = props.app.contract;

  useEffect(() => {
    setLoader(true);
    contract.profile_get({profile_id: profileId}).then((profile) => {
      setProfile(profile);
      setLoader(false);
    });
  }, []);

  useEffect(() => {
    setLoader(true);
    contract.lot_list_offering_by({profile_id: profileId}).then((lots) => {
      setLotsOffering(lots);
      setLoader(false);
    });
  }, []);

  useEffect(() => {
    setLoader(true);
    contract.lot_list_bidding_by({profile_id: profileId}).then((lots => {
      setLotsBidding(lots);
      setLoader(false);
    }));
  }, []);

  const claim = async () => {
    setClaimLoader(true);
    try {
      await contract.profile_rewards_claim({}, BOATLOAD_OF_GAS).then(() => {
        contract.profile_get({profile_id: profileId}).then((profile) => {
          setProfile(profile);
          console.log(profile);
          setClaimLoader(false);
        });
      });
    } catch (e) {
      console.error(e);
      setClaimLoader(false);
    }
  };


  return (
    <div>
    { loader ? <Loader/> : profile ?
      <div className="mt-3">
        <h5><strong>{profileId}</strong></h5>
        <p><strong>available:</strong> {nearTo(profile.rewards_available)}<span title="NEAR Tokens">Ⓝ</span> <strong>claimed:</strong> {nearTo(profile.rewards_claimed)}<span title="NEAR Tokens">Ⓝ</span></p>
        {claimLoader ? <Loader position={'left'}/> : <button name="claim_rewards" className="mb-5" disabled={!parseFloat(profile.rewards_available)} onClick={(e) => claim()}>Claim rewards</button> }
        {lotsOffering.length ? <h5>My Lots offer</h5> : ''}
        { loader ?
          <Loader key={'1'}/> :
          <ul className="lot_list">
            {lotsOffering.map((lot, i) =>
              <Lot lot={lot} key={lot.lot_id} contract={contract} currentUser={profileId}/>
            )}
          </ul>
        }

        {lotsBidding.length ? <h5>My Lots bidding</h5> : ''}
        { loader ?
          <Loader key={'2'}/> :
          <ul className="lot_list">
            {lotsBidding.map((lot, i) =>
              <Lot lot={lot} key={lot.lot_id} contract={contract} currentUser={profileId}/>
            )}
          </ul>
        }
      </div> :
      <div>Profile not found</div> }</div>
  );


}

export default Profile
