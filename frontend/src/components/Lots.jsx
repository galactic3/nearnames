import React from 'react';
import PropTypes from 'prop-types';
import utils from '../utils';
import Big from 'big.js';

export default function Lots({ lots }) {
  return (
    <>
      <h2>Lots</h2>
      <ul>
      {lots.map((lot, i) =>
        // TODO: format as cards, add timestamp
        <li key={i} className='lot_item'>
          <ul>
            <li>Lot name: <strong>{lot.lot_id}</strong></li>
            <li>Seller name: <strong>{lot.seller_id}</strong></li>
            <li>Current price: <strong>{Big(lot.reserve_price).div(10 ** 24).toFixed(2)}</strong></li>
            <li>Lot expired: <strong>{new Date(Math.floor(lot.finish_timestamp/1000000)).toUTCString()}</strong></li>
          </ul>
          <div class="button_wrapper">
            <input type="number" id="bid_price"/><button>Bid</button>
            <button>Buy now</button>
          </div>
        </li>
      )}
      </ul>
    </>
  );
}

Lots.propTypes = {
  lots: PropTypes.array
};
