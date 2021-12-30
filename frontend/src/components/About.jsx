import React, {useState} from 'react';

function AboutPage () {

  const [clickImage, setClickImage] = useState(false);

  const onClickImage = () => {
    setClickImage(!clickImage);
  };

  return (
    <div className="container about">
      <h2>About Near Names</h2>
      <p>Near Names is a smart contract and application allowing users to safely sell and buy near accounts (and more importantly their respective domain names).</p>

      <h3>Auction flow</h3>

      <img className={clickImage ? 'zoom' : ''} src="https://raw.githubusercontent.com/wiki/galactic3/nearnames/images/auction_flow_success.png" onClick={() => onClickImage()} alt="auction_flow_success"/>

      <p>The sale process is an auction, the account offering the highest price wins. Read more in <a target="_blank" href="https://github.com/galactic3/nearnames/wiki/Auction-flow">Auction Flow</a></p>

      <h3>Prior art</h3>

      <p>There already exists <a target="_blank" href="https://near.bet">near.bet</a> accounts auction marketplace.
      This project brings more flexibility to the auction process.</p>

      <h3>Key features</h3>

      <ul className="default">
        <li>Custom bid amount: no fixed bid step, bidder can place arbitrary high bid right from the start.</li>
        <li>Adjustable duration: after auction time ends, either lot is sold, or returned back to owner.</li>
        <li>Lot withdrawal: lot can be withdrawn from auction if there were no bids yet.</li>
        <li>Reserve price: minimum amount for the bid</li>
        <li>Buy now price: price for which lot will be sold instantly.</li>
      </ul>

      <h3>Marketplace contract documentation</h3>

      <p>Detailed description of marketplace contract is in <a target="_blank" href="https://github.com/galactic3/nearnames/wiki/Marketplace-contract">Marketplace contract</a></p>
    </div>
  )
}

export default AboutPage;