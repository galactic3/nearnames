import React, { useState, useEffect } from 'react';
import LotsList from "./LotsList";
import { InputAdornment, TextField } from "@mui/material";
import SearchIcon from '@mui/icons-material/Search';



function Lots(props) {

  const contract = props.app.contract;

  const getLots = async () => {
    setLoader(true);
    await contract.lot_list().then((lots) => {
      setCashLots(lots);
      setLots(lots);
    })
    setLoader(false);
  }

  const filterList = async (e) => {
    const value = e.target.value.toLowerCase();
    if(value !== '') {
      const result = lots.filter((lot) => {
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
    <div>
      <TextField
        onChange={(e) => filterList(e)} value={filter}
        type="search" variant="standard" id="filterList"
        sx={{ mb: 3, width: 350 }}
        InputProps={{
          startAdornment: (
            <InputAdornment position="start">
              <SearchIcon />
            </InputAdornment>
          ),
        }}/>
      <LotsList lots={lots} getLots={getLots} loader={loader} {...props} />
    </div>
  );
}

export default Lots;
