FROM node:24-alpine

ENV NODE_ENV=production

RUN apk add --no-cache \
    libstdc++ \
    libgcc \
    tzdata

WORKDIR /app


COPY ./package.json /app/package.json

# Installing the equivalents of requests, pandas, numpy, and dateutil
# axios = requests
# danfojs-node = pandas
# mathjs = numpy (general math/matrices)
# date-fns = python-dateutil
RUN npm install axios danfojs-node mathjs date-fns

COPY ./sdk.js /app/sdk.js
COPY ./bootstrap.js /app/bootstrap.js

USER node

CMD ["node", "bootstrap.js"]