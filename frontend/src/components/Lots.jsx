import React, { useState, useEffect } from 'react';
import LotsList from "./LotsList";
import SearchIcon from '@mui/icons-material/Search';
import ls from 'local-storage'



function Lots(props) {

  const contract = props.app.contract;

  const notSafeLots = ls.get('NotSafeLots') || '';

  const getLots = async () => {
    setLoader(true);
    await contract.lot_list().then((lots) => {
      const result = lots.filter((lot) => {
        if (notSafeLots.includes(lot.lot_id)) {
          lot.status = 'NotSafe';
        }
        return lot.status === 'OnSale';
      })
      setCashLots(result);
      setLots(result);
    })
    setLoader(false);
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
        <input type="text" className="search" onChange={(e) => filterList(e)} value={filter}/>
        {filter && <span className="search-result">{lots.length} results <b>"{filter}"</b> found</span>}
      </div>
      <LotsList lots={lots} getLots={getLots} showStatus={false} loader={loader} {...props} />
    </div>
  );
}

export default Lots;
