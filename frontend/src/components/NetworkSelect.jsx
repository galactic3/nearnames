import React, {useState} from 'react';
import {MenuItem, Select} from "@mui/material";
import KeyboardArrowDownRoundedIcon from "@mui/icons-material/KeyboardArrowDownRounded";

function NetworkSelect () {

  const [network, setNetwork] = useState('testnet');

  const onNetworkChange = (e) => {
    const value = e.target.value;
    setNetwork(value);
  }

  return (
    <div className="network-select">
      <Select
        labelId="network-select-label"
        id="network-select"
        value={network}
        onChange={onNetworkChange}
        IconComponent={KeyboardArrowDownRoundedIcon}
      >
        <MenuItem value='testnet'>Testnet</MenuItem>
        <MenuItem value='mainnet' disabled={true}>Mainnet (coming soon)</MenuItem>
      </Select>
    </div>
  )
}

export default NetworkSelect;