import React from 'react';
import {Box, IconButton, Modal} from "@mui/material";
import CloseIcon from '@mui/icons-material/Close';

function ModalConfirm (props) {

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
        <h3>Warning</h3>
        <div className="alert-content">{props.content}</div>
        <div className="button_wrapper">
          <button className="outlined" onClick={() => props.onClose()}>Cancel</button>
          <button className="" onClick={() => props.onClose()}>Continue</button>
        </div>
      </Box>
    </Modal>
  )
}

export default ModalConfirm;