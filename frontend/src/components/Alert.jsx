import React from 'react';
import {Box, IconButton, Modal} from "@mui/material";
import CloseIcon from '@mui/icons-material/Close';

function ModalAlert (props) {

  return (
    <Modal {...props}>
      <Box className="modal-container alert_modal">
      <IconButton
        aria-label="close"
        onClick={props.onClose}
        className="button-icon"
        sx={{
          position: 'absolute',
          right: 8,
          top: 8,
          color: 'var(--gray)',
        }}
      >
        <CloseIcon />
      </IconButton>
        <h3>Error</h3>
        <div className="alert-content">{props.content}</div>
        <button className="alert-button full-width" onClick={() => props.onClose()}>Continue</button>
      </Box>
    </Modal>
  )
}

export default ModalAlert;