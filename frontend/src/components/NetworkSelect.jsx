import React, {useState} from 'react';
import {MenuItem, Select} from "@mui/material";
import KeyboardArrowDownRoundedIcon from "@mui/icons-material/KeyboardArrowDownRounded";

function NetworkSelect () {

  const network = process.env.NETWORK_ID;
  const host = process.env.PUBLIC_URL;

  const onNetworkChange = (e) => {
    const value = e.target.value;
    const sub = value === 'testnet' ? 'testnet.' : '';
    window.location.replace('https://' + sub + host);
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
        <MenuItem value='mainnet'>MAINNET</MenuItem>
        <MenuItem value='testnet'>TESTNET</MenuItem>
      </Select>
    </div>
  )
}

export default NetworkSelect;