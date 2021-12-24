import React, {useState} from 'react';
import ModalOffer from "./Offer";

function CreateOffer(props) {

  const [createOfferModalShow, setCreateOfferModalShow] = useState(false)

  const createOffer = () => {
    setCreateOfferModalShow(true);
  }

  return (
    <div className="create_offer">
      <button onClick={createOffer}>Create new offer</button>
      <ModalOffer
        {...props}
        open={createOfferModalShow}
        onClose={() => setCreateOfferModalShow(false)}
      />
    </div>
  )

}

export default CreateOffer;