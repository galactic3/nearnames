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
    contract.profile_rewards_claim({}, BOATLOAD_OF_GAS).then(() => {
        setProfile(profile);
      }
    );
  };


  return (
    <div>
    { loader ? <Loader/> : profile ?
      <div>
        <h2>{profileId}</h2>
        <h5>Rewards</h5>
        <p><strong>available:</strong> {nearTo(profile.rewards_available)}<span title="NEAR Tokens">Ⓝ</span> <strong>claimed:</strong> {nearTo(profile.rewards_claimed)}<span title="NEAR Tokens">Ⓝ</span></p>
        <p><button name="claim" disabled={!!profile.rewards_available} onClick={(e) => claim()}>Claim rewards</button></p>
        <h5>My Lots offer</h5>
        { loader ?
          <Loader/> :
          <ul className="lot_list">
            {lotsOffering.map((lot, i) =>
              <Lot lot={lot} key={lot.lot_id} currentUser={profileId}/>
            )}
          </ul>
        }

        <h5>My Lots bidding</h5>
        { loader ?
          <Loader/> :
          <ul className="lot_list">
            {lotsBidding.map((lot, i) =>
              <Lot lot={lot} key={lot.lot_id} currentUser={profileId}/>
            )}
          </ul>
        }
      </div> :
      <div>Profile not founded</div> }</div>
  );


}

export default Profile
