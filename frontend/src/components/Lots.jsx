import React, { useState, useEffect } from 'react';
import LotsList from "./LotsList";



function Lots(props) {

  const contract = props.app.contract;

  const getLots = async () => {
    setLoader(true);
    await contract.lot_list().then(setLots);
    setLoader(false);
  }

  const [lots, setLots] = useState([]);
  const [loader, setLoader] = useState(false);

  useEffect(async () => {
    await getLots();
  }, []);

  return (
    <LotsList lots={lots} getLots={getLots} loader={loader} {...props} />
  );
}

export default Lots;
