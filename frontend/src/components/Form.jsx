import React from 'react';
import PropTypes from 'prop-types';
import Big from 'big.js';

export default function Form({ onSubmit, currentUser }) {
  return (
    <form onSubmit={onSubmit}>
      <fieldset id="fieldset">
        <p>Hello, { currentUser.accountId }!</p>
        <p className="highlight">
          <label htmlFor="seller_id">Seller account:</label>
          <input
            autoComplete="off"
            autoFocus
            id="seller_id"
            required
          />
        </p>
        <p>
          <label htmlFor="reserve_price">Min price:</label>
          <input
            autoComplete="off"
            defaultValue="1.5"
            id="reserve_price"
            min="1.5"
            step="0.01"
            type="number"
            required
          />
          <span title="NEAR Tokens">Ⓝ</span>
        </p>
        <p>
          <label htmlFor="buy_now_price">Buy now price:</label>
          <input
            autoComplete="off"
            id="buy_now_price"
            min="1.5"
            step="0.01"
            type="number"
            required
          />
          <span title="NEAR Tokens">Ⓝ</span>
        </p>
        <p>
          <label htmlFor="buy_now_price">Duration in hours:</label>
          <input
            autoComplete="off"
            id="duration"
            type="number"
          />
        </p>
        <button type="submit">
          Create offer
        </button>
      </fieldset>
    </form>
  );
}

Form.propTypes = {
  onSubmit: PropTypes.func.isRequired,
  currentUser: PropTypes.shape({
    accountId: PropTypes.string.isRequired,
    balance: PropTypes.string.isRequired
  })
};
