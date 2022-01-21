import React from 'react'
import { CircularProgress } from "@mui/material";

const Loader = (props) => {
  return (
    <div className="spinner">
      <CircularProgress sx={{
        color: 'var(--link)',
        margin: 'auto',
        ...props
      }}/>
    </div>
  )
}

export default React.memo(Loader);