import React, { useState } from 'react';
import LotsList from "./LotsList";
import SearchIcon from '@mui/icons-material/Search';
import {fetchBidSafety, loadListPaginated} from '../utils';
import {ToggleButton, ToggleButtonGroup} from "@mui/material";
import {isDesktop} from "react-device-detect";

function Lots(props) {

  const contract = props.contract;

  const initStatus = ['OnSale', 'SaleSuccess'];

  const getLots = async () => {

    let result = [];

    setLoader(true);

    console.time('lots fetch');

    await loadListPaginated(args => contract.lot_list(args)).then(async (lots) => {
      result = lots.filter((lot) => {
        return initStatus.includes(lot.status);
      });
      setCashLots(result);
      setLots(result);
    })
    setLoader(false);

    console.timeEnd('lots fetch');

    console.time('lots check');

    await Promise.all(result.map(async (l) => {
      const isSafe = await fetchBidSafety(l.lot_id, props.near, props.nearConfig);
      l.notSafe = !isSafe;
    }));

    setLots([...result]);

    console.timeEnd('lots check');

  }

  const putLot = async (lot) => {
    const isSafe = await fetchBidSafety(lot.lot_id, props.near, props.nearConfig);
    lot.notSafe = !isSafe;
    const updatedLots = lots.map((l) => {
      if (l.lot_id === lot.lot_id) {
        return lot;
      }
      return l;
    });
    setLots(updatedLots);
    const updatedCashLots = cashLots.map((l) => {
      if (l.lot_id === lot.lot_id) {
        return lot;
      }
      return l;
    });
    setCashLots(updatedCashLots);
  }

  const filterList = async (e) => {
    const value = e.target.value.toLowerCase();
    const updatedLots = cashLots.filter((lot) => {
      return (status ? status : initStatus).includes(lot.status) && lot.lot_id.toLowerCase().includes(value);
    })
    setLots(updatedLots);
    setFilter(value);
  }

  const handleChangeStatus = (e, value) => {
    const updatedLots = cashLots.filter((lot) => {
      return (value ? value : initStatus).includes(lot.status) && lot.lot_id.toLowerCase().includes(filter);
    })
    setLots(updatedLots);
    setStatus(value);
  };

  const [lots, setLots] = useState([]);
  const [cashLots, setCashLots] = useState([]);
  const [filter, setFilter] = useState('');
  const [status, setStatus] = useState('');
  const [loader, setLoader] = useState(false);

  return (
    <div className="container">
      <div className="search-wrapper">
        <SearchIcon className="search-icon"/>
        <input type="text" className="search" placeholder="Search lots for sale" onChange={(e) => filterList(e)} value={filter}/>
        {filter && <span className="search-result">{lots.length} results {isDesktop && <strong>"{filter}"</strong>} found</span>}
      </div>
      <div className="status-wrapper">
        <ToggleButtonGroup
          exclusive
          value={status}
          onChange={handleChangeStatus}
        >
          <ToggleButton value="OnSale">On sale</ToggleButton>
          <ToggleButton value="SaleSuccess">Finished</ToggleButton>
        </ToggleButtonGroup>
      </div>
      <LotsList lots={lots} getLots={getLots} putLot={putLot} signIn={props.signIn} showStatus={true} loader={loader} {...props} />
    </div>
  );
}

export default Lots;
