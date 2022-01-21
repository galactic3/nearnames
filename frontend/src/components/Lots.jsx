import React, { useState } from 'react';
import LotsList from "./LotsList";
import SearchIcon from '@mui/icons-material/Search';
import {fetchBidSafety, loadListPaginated} from '../utils';

function Lots(props) {

  const contract = props.contract;

  const getLots = async () => {

    let result = [];

    setLoader(true);

    console.time('lots fetch');

    await loadListPaginated(args => contract.lot_list(args)).then(async (lots) => {
      result = lots.filter((lot) => {
        return lot.status === 'OnSale';
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
    }).filter((l) => {
      return l.status === 'OnSale';
    })
    setCashLots(updatedLots);
    setLots(updatedLots);
  }

  const filterList = async (e) => {
    const value = e.target.value.toLowerCase();
    if(value !== '') {
      const result = cashLots.filter((lot) => {
        return lot.lot_id.toLowerCase().includes(value);
      })
      setLots(result);
    } else {
      setLots(cashLots);
    }
    setFilter(value);
  }

  const [lots, setLots] = useState([]);
  const [cashLots, setCashLots] = useState([]);
  const [filter, setFilter] = useState('');
  const [loader, setLoader] = useState(false);

  return (
    <div className="container">
      <div className="search-wrapper">
        <SearchIcon className="search-icon"/>
        <input type="text" className="search" placeholder="Search lots for sale" onChange={(e) => filterList(e)} value={filter}/>
        {filter && <span className="search-result">{lots.length} results <strong>"{filter}"</strong> found</span>}
      </div>
      <LotsList lots={lots} getLots={getLots} putLot={putLot} signIn={props.signIn} showStatus={false} loader={loader} {...props} />
    </div>
  );
}

export default Lots;
