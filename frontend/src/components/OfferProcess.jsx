import React from 'react'
import Alert from "@mui/material/Alert";
import {CircularProgress} from "@mui/material";
import { Redirect } from "react-router-dom";
import Link from "react-router-dom/es/Link";

function OfferProcessPage (props) {

  let finished = props.offerFinished;
  let success = props.offerSuccess;
  let active = props.offerActive;
  let offerFailureReason = props.offerFailureReason;
  let offerSuccessMessage = props.offerSuccessMessage;

  if (!active) {
    return (<Redirect push to="/"/>)
  }

  return (
    <div className='container offer-container'>
      {finished
        ? (success
          ? (
            <Alert className="alert-container" severity="success">Success! {offerSuccessMessage} Back to <Link to='/'>Homepage</Link></Alert>
          ) : (
            <Alert className="alert-container" severity="error">
              Something went wrong, prease refresh the page.
              <br/>
              {offerFailureReason && <span>Failure reason: {offerFailureReason}.</span>}
            </Alert>
          )
        ) : (
          <div>

            <Alert className="alert-container" severity="warning">Do not refresh or close the page</Alert>

            <Alert className="alert-container" severity="info">It may take up to 5 minutes to complete</Alert>

            <ul className="offer-process-list">
              {props.offerProcessOutput.map((msg, i) =>
                <li key={i}>{msg}</li>
              )}
            </ul>

            <div className="spinner">
              <CircularProgress sx={{
                color: 'var(--link)',
                margin: 'auto'
              }}/>
            </div>

          </div>
        )}

    </div>
  )
}

export default OfferProcessPage
