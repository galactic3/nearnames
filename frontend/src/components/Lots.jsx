import React, { useState, useEffect } from 'react';
import LotsList from "./LotsList";
import SearchIcon from '@mui/icons-material/Search';
import ls from 'local-storage';
import { loadListPaginated } from '../utils';

function Lots(props) {

  const contract = props.contract;

  const notSafeLots = ls.get('NotSafeLots') || '';

  const getLots = async () => {
    setLoader(true);

    await loadListPaginated(args => contract.lot_list(args)).then((lots) => {
      const result = lots.filter((lot) => {
        if (notSafeLots.includes(lot.lot_id)) {
          lot.notSafe = true;
        }
        return lot.status === 'OnSale';
      })
      setCashLots(result);
      setLots(result);
    })
    setLoader(false);
  }

  const putLot = (lot) => {
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

  useEffect(async () => {
    await getLots();
  }, []);

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
