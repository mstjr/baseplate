FROM node:24-alpine

ENV NODE_ENV=production

RUN apk add --no-cache \
    libstdc++ \
    libgcc \
    tzdata

WORKDIR /app


COPY ./package.json /app/package.json

RUN npm install axios danfojs-node mathjs date-fns

COPY ./sdk.js /app/sdk.js
COPY ./bootstrap.js /app/bootstrap.js

RUN mkdir /output
RUN chown runner:runner /output

USER runner

CMD ["node", "bootstrap.js"]